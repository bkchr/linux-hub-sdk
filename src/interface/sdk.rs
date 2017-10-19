use std::fs::Permissions;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::thread;

use geeny_api;
use log;
use mvdb::Mvdb;

use auth_manager::{self, ServiceCredentials};
use errors::*;
use interface::config::HubSDKConfig;
use things_db::{self, PartialThingMessage, ThingDb};

/// Interface handle for a `HubSDK` instance
#[derive(Clone)]
pub struct HubSDK {
    config: HubSDKConfig, // Do I need to hold this? Should it be Arc?
    thing_db_data: Mvdb<ThingDb>,
    thing_db_handle: Arc<JoinHandle<()>>, // TODO: Not very useful without a way to join
    auth_mgr_handle: Arc<JoinHandle<()>>, // TODO: Not very useful without a way to join
    credentials: Mvdb<ServiceCredentials>,
}

impl HubSDK {
    /// Create a new instance of the Geeny Hub SDK. SDK will immediately
    /// begin operation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// extern crate hub_sdk;
    ///
    /// use hub_sdk::{HubSDK, HubSDKConfig};
    ///
    /// fn main() {
    ///     let sdk_cfg = HubSDKConfig::default();
    ///
    ///     // Begin running the SDK. The hub_sdk handle may be used to interact with
    ///     // the sdk. This handle may be cloned and given to multiple consumers
    ///     let hub_sdk = HubSDK::new(sdk_cfg);
    /// }
    /// ```
    pub fn new(cfg: HubSDKConfig) -> Self {
        // Create relevant folders before proceeding (otherwise further steps may fail)
        make_dirs(&cfg).expect("Failed to create required folders");

        let credentials: Mvdb<ServiceCredentials> =
            Mvdb::from_file_or_default(&cfg.geeny_creds_file)
                .expect("Failed to load/create credentials file");

        // Make sure permissions are correct for credentials file
        let creds_perm = Permissions::from_mode(0o600);
        fs::set_permissions(cfg.geeny_creds_file.clone(), creds_perm)
            .expect("Failed to set credentials file permissions");

        // Create accessors for config data
        let auth_mgr_cfg = cfg.clone();
        let runner_cfg = cfg.clone();
        let auth_mgr_auth = credentials.clone();
        let runner_auth = credentials.clone();

        let mut dbr = things_db::ThingDbRunner::new(runner_cfg, runner_auth);
        let data = dbr.thing_db_handle();

        let auth_mgr = thread::spawn(move || {
            auth_manager::auth_manager(auth_mgr_cfg, &auth_mgr_auth);
        });
        let tdb_run = thread::spawn(move || { dbr.run(); });


        Self {
            config: cfg,
            thing_db_data: data,
            thing_db_handle: Arc::new(tdb_run),
            auth_mgr_handle: Arc::new(auth_mgr),
            credentials: credentials,
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // AUTH
    ///////////////////////////////////////////////////////////////////////////

    // TODO return type
    /// Check whether a given token is still valid.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use hub_sdk::{HubSDK, HubSDKConfig};
    /// let sdk_cfg = HubSDKConfig::default();
    /// let hub_sdk = HubSDK::new(sdk_cfg);
    ///
    /// let (email, valid) = hub_sdk.check_token()
    ///     .expect("Failed to access auth info");
    ///
    /// println!("Username: {}, Valid Token: {}", email, valid);
    /// ```
    pub fn check_token(&self) -> Result<(String, bool)> {
        use geeny_api::models::AuthLoginResponse;

        // Do we have a token at all now?
        let (email, tkn_maybe) = self.credentials.access(|db| {
            let email = db.username.clone();
            let tkn_maybe = db.token.clone();
            (email, tkn_maybe)
        })?;

        let valid = if let Some(tkn) = tkn_maybe {
            let tkn_req = AuthLoginResponse { token: tkn };

            // Does the token check out?
            // TODO - difference between bad token and network error?
            self.config.connect_api.check_token(&tkn_req).is_ok()
        } else {
            false
        };

        Ok((email, valid))
    }

    /// Perform a login to the Geeny API, allowing further operations
    /// such as creating a Thing
    ///
    /// # NOTE
    ///
    /// Currently the SDK performs a login using Basic Authentication. The SDK
    /// uses the Email and Password to log in and retrieve an access token. The
    /// SDK DOES NOT store the password. The SDK will attempt to refresh the
    /// access token periodically when running, however after a long period
    /// of not running (e.g., when the service is stopped, or the Hub has been
    /// powered down), a new login will need to be performed.
    ///
    /// The `HubSDK::check_token()` method may be used to check whether a new
    /// login needs to be performed.
    ///
    /// It is HIGHLY RECOMMENDED never to store the user's password, and instead
    /// prompt the user directly whenever a login is necessary.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use hub_sdk::{HubSDK, HubSDKConfig};
    /// let sdk_cfg = HubSDKConfig::default();
    /// let hub_sdk = HubSDK::new(sdk_cfg);
    ///
    /// hub_sdk.login("cool_username@email.com", "S3cure_P@ssw0rd")
    ///     .expect("Failed to log in!");
    /// ```
    pub fn login(&self, email: &str, password: &str) -> Result<()> {
        use geeny_api::models::AuthLoginRequest;

        let rqst = AuthLoginRequest {
            email: email.into(),
            password: password.into(),
        };

        if let Ok(token) = self.config.connect_api.login(&rqst) {
            self.credentials.access_mut(move |db| {
                db.username = email.into();
                db.token = Some(token.token);
            })?;
            return Ok(());
        }

        bail!("failure")
    }

    /// Logout of the Geeny API. No further API operations will be possible
    /// until a Login occurs. All active devices will be immediately unpaired
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use hub_sdk::{HubSDK, HubSDKConfig};
    /// let sdk_cfg = HubSDKConfig::default();
    /// let hub_sdk = HubSDK::new(sdk_cfg);
    ///
    /// hub_sdk.logout().expect("Failed to log out!");
    /// ```
    pub fn logout(&self) -> Result<()> {
        // TODO: We probably need to do some more stuff on logout, like:
        //   - Stopping the auth manager
        //   - Maybe offer to delete all devices?
        self.credentials.access_mut(move |db| {
            db.username = "".into();
            db.token = None;
        })?;

        self.thing_db_data
            .access_mut(|db| db.unpair_all())?;

        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////
    // Things
    ///////////////////////////////////////////////////////////////////////////
    /// Create a new thing on the Geeny cloud
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// extern crate uuid;
    /// use uuid::Uuid;
    /// use hub_sdk::{HubSDK, HubSDKConfig};
    /// use hub_sdk::geeny_api::models::ThingRequest;
    /// let sdk_cfg = HubSDKConfig::default();
    /// let hub_sdk = HubSDK::new(sdk_cfg);
    ///
    /// let new_thing = ThingRequest {
    ///     name: "New Demo Thing".into(),
    ///     serial_number: "ABC123456".into(),
    ///     thing_type: Uuid::from("2CB7F29A-527B-11E7-B114-B2F933D5FE66"),
    /// };
    ///
    /// hub_sdk.create_thing(new_thing)
    ///     .expect("Failed to create new thing!");
    /// ```
    pub fn create_thing(&self, request: geeny_api::models::ThingRequest) -> Result<()> {
        self.thing_db_data
            .access_mut(|db| db.add_thing(request))?
            .chain_err(|| "Failed to add thing")
    }

    /// Delete a thing from the Geeny cloud. The thing must not be
    /// currently active, e.g., it must first be unpaired
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use hub_sdk::{HubSDK, HubSDKConfig};
    /// let sdk_cfg = HubSDKConfig::default();
    /// let hub_sdk = HubSDK::new(sdk_cfg);
    ///
    /// hub_sdk.delete_thing_by_serial("ABC123456")
    ///     .expect("Failed to delete thing!");
    /// ```
    pub fn delete_thing_by_serial(&self, serial: &str) -> Result<()> {
        let exists = self.thing_db_data
            .access_mut(|db| db.contains_serial(serial))?;

        if exists {
            bail!("Device must be unpaired before deletion")
        }

        let token = self.credentials
            .access(|auth| auth.token.clone())?
            .ok_or_else(|| Error::from("No token, cannot delete. Please log in"))?;

        let thing = match self.config.api.get_thing_by_serial(&token, &serial)? {
            Some(t) => t,
            None => {
                // If there is no matching device, still report okay
                return Ok(());
            }
        };

        let _ = self.config.api.delete_thing(&token, &thing.id)?;

        Ok(())
    }

    /// Unpair a thing that is managed by the SDK. Unpairing a thing not
    /// currently managed by the SDK will not cause an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use hub_sdk::{HubSDK, HubSDKConfig};
    /// let sdk_cfg = HubSDKConfig::default();
    /// let hub_sdk = HubSDK::new(sdk_cfg);
    ///
    /// hub_sdk.unpair_thing_by_serial("ABC123456")
    ///     .expect("Failed to unpair thing!");
    /// ```
    pub fn unpair_thing_by_serial(&self, serial: &str) -> Result<()> {
        self.thing_db_data.access_mut(|db| {
            // respond OK to bad unpair requests
            if let Err(e) = db.unpair(serial) {
                log::warn!("unpairing: unknown device, {}", e);
            }
        })?;

        Ok(())
    }

    /// Send messages to the Geeny cloud on behalf of a thing
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use hub_sdk::{HubSDK, HubSDKConfig};
    /// use hub_sdk::services::PartialThingMessage;
    ///
    /// let sdk_cfg = HubSDKConfig::default();
    /// let hub_sdk = HubSDK::new(sdk_cfg);
    ///
    /// let messages = vec!(
    ///     PartialThingMessage {
    ///         topic: "demo/send/path".into(),
    ///         msg: "demonstration message".into(),
    ///     },
    ///     PartialThingMessage {
    ///         topic: "demo/other/path".into(),
    ///         msg: "second demonstration message".into(),
    ///     },
    /// );
    ///
    /// hub_sdk.send_messages("ABC123456", &messages)
    ///     .expect("Failed to send messages!");
    /// ```
    pub fn send_messages(&self, serial: &str, messages: &[PartialThingMessage]) -> Result<()> {
        self.thing_db_data
            .access_mut(|db| db.hub_tx(serial, messages))??;

        Ok(())
    }

    /// Obtain any messages sent from the Geeny cloud to a given thing
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use hub_sdk::{HubSDK, HubSDKConfig};
    /// let sdk_cfg = HubSDKConfig::default();
    /// let hub_sdk = HubSDK::new(sdk_cfg);
    ///
    /// let messages = hub_sdk.receive_messages("ABC123456")
    ///     .expect("Failed to receive messsages!");
    ///
    /// for msg in messages {
    ///     println!("topic: >>{}<<, message: >>{}<<", msg.topic, msg.msg);
    /// }
    /// ```
    pub fn receive_messages(&self, serial: &str) -> Result<Vec<PartialThingMessage>> {
        self.thing_db_data.access_mut(|db| db.hub_rx(serial))?
    }
}

fn make_dirs(cfg: &HubSDKConfig) -> Result<()> {
    let folder_perms = Permissions::from_mode(0o755);
    let paths = vec![
        // Get the folder the element file resides in
        cfg.element_file
            .parent()
            .ok_or_else(|| Error::from("bad element path spec"))?,

        // Get the folder the credentials file resides in
        cfg.geeny_creds_file
            .parent()
            .ok_or_else(|| Error::from("bad credentials path spec"))?,

        // The folder for MQTT certificates
        &cfg.mqtt_cert_path,
    ];

    for path in paths {
        fs::create_dir_all(path)
            .chain_err(|| "failed to create certificate directory")?;
        fs::set_permissions(path, folder_perms.clone())
            .chain_err(|| "Couldn't set permissions")?;
    }


    Ok(())
}
