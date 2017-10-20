use geeny_api::{ThingsApi, ConnectApi};
use std::path::PathBuf;

/// Configuration structure for a `HubSDK` instance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HubSDKConfig {
    /// Connection object for the main Things API, e.g., `https://labs.geeny.io`
    pub api: ThingsApi,

    /// Connection object for the authorization API, e.g., `https://connect.geeny.io`
    pub connect_api: ConnectApi,

    /// Path to a file to store information regarding Things/Elements that have been
    /// paired with the SDK. This file stores sensitive information such as Certificate
    /// Pairs for each device
    pub element_file: PathBuf,

    /// Path to a file to store information regarding the user currently authorized to
    /// use this Hub. This file stores sensitive information such as the username/email
    /// of the current user, as well as the current API token used to make device management
    /// requests
    pub geeny_creds_file: PathBuf,

    /// Path to a folder to store certificates used to connect via MQTT. This folder stores
    /// sensitive information, such as private and public key pairs for each device paired
    /// with the SDK
    pub mqtt_cert_path: PathBuf,

    // TODO: maybe consolodate these into an "mqtt" struct,
    // TODO DI-235 - SocketAddr
    /// The MQTT host to connect to all devices, e.g., `mqtt.geeny.io`
    pub mqtt_host: String,

    /// The MQTT port to connect to all devices, e.g., `8883`
    pub mqtt_port: u16,
}

impl Default for HubSDKConfig {
    /// Create a Configuration Structure for the `HubSDK`
    ///
    /// # Example
    ///
    /// ```rust
    /// use hub_sdk::HubSDKConfig;
    ///
    /// let sdk_cfg = HubSDKConfig::default();
    /// ```
    fn default() -> Self {
        HubSDKConfig {
            api: ThingsApi::default(),
            connect_api: ConnectApi::default(),

            // todo get cwd?
            element_file: PathBuf::from("/tmp/elements.mvdb.json"),
            geeny_creds_file: PathBuf::from("/tmp/credentials.mvdb.json"),
            mqtt_cert_path: PathBuf::from("/tmp/geeny_certificates"),

            mqtt_host: "mqtt.geeny.io".into(),
            mqtt_port: 8883,
        }
    }
}
