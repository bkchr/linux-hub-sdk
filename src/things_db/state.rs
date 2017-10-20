// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::fs::{self, File, Permissions};
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::mpsc::{Sender, TryIter};
use std::sync::{Arc, Mutex};

use log;
use rumqtt::{self, MqttClient};

use errors::*;
use geeny_api::ThingsApi;
use geeny_api::models::{Resource, ResourceMethod, Thing, ThingRequest};
use things_db::PartialThingMessage;

/// `ThingSyncState` is a three part state machine. The three states are:
///   * `Created`: We have received a local request to create a Geeny
///       Thing. We may now handle incoming messages for that Thing, but
///       the Thing has not yet been created (or found) on the Geeny Cloud
///   * `GatheringMetadata`: The Thing has been either found or created on the
///       Geeny Cloud, however we still need additional data before establishing
///       a data stream connection (e.g., MQTT)
///   * `Active`: All necessary information has been gathered. The Thing will attempt
///       to establish and maintain a data stream connection
///
/// State transitions from one `ThingSyncState` to another occur by returning an
///   Option<Self>, where Some(Self) notes that the transition was successful,
///   and the current state should be replaced with a new one
#[derive(Serialize, Deserialize)]
pub enum ThingSyncState {
    Created(ThingRequest),
    GatheringMetadata(Thing),
    Active(MetaThing),
}

impl fmt::Display for ThingSyncState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ThingSyncState::*;
        match *self {
            Created(ref t) => write!(f, "Created: s/n: {}", t.serial_number),
            GatheringMetadata(ref t) => {
                write!(f, "Thing: s/n: {}, gtid: {}", t.serial_number, t.id)
            }
            Active(ref t) => write!(
                f,
                "MetaThing: s/n: {}, gtid: {}",
                t.thing.serial_number,
                t.thing.id
            ),
        }
    }
}

impl ThingSyncState {
    /// We have received a request to create a device, attempt to register it on the
    /// Geeny cloud
    pub fn create_new_thing(
        api: &ThingsApi,
        token: &str,
        thing_request: &ThingRequest,
    ) -> Option<Self> {
        // First check if there is an existing device matching this serial number
        let existing = api.get_thing_by_serial(token, &thing_request.serial_number);

        match existing {
            Ok(Some(_new_thing)) => {
                // TODO: restore after we can retrieve certs for existing devices
                // https://jira.geeny.io/browse/DI-211
                log::error!(
                    "Found existing thing with SN:{}, creating new device anyway",
                    thing_request.serial_number
                );
                // return Some(ThingInstance::from_thing(new_thing))
            }
            Ok(None) => {}
            Err(e) => {
                log::error!("Failed to query existing things: {}", e);
                return None;
            }
        };

        // Doesn't exist, make a new one
        match api.create_thing(token, thing_request) {
            Ok(new_thing) => {
                // Transition from Created to GatheringMetadata
                Some(ThingSyncState::GatheringMetadata(new_thing))
            }
            Err(e) => {
                log::error!("Failed to create thing: {:?}", e);
                // No transition on failure
                None
            }
        }
    }

    /// We have created the device in the Geeny cloud, we now need to get some
    /// associated metadata before we can establish an MQTT connection
    pub fn gather_thing_metadata(
        api: &ThingsApi,
        token: &str,
        geeny_thing: &Thing,
    ) -> Option<Self> {
        // get all resources for this thing type
        let meta_req = api.get_thing_type_resources(token, &geeny_thing.thing_type);

        // if we got all the data we needed, we can transition state
        if let Ok(meta) = meta_req {
            Some(ThingSyncState::Active(MetaThing {
                thing: geeny_thing.clone(),
                resources: meta,
                mqtt_handle: None,
                ca_file_name: None,
                cert_file_name: None,
                key_file_name: None,
            }))
        } else {
            // No transition on failure
            None
        }
    }

    pub fn consume(self) {
        // Disconnect and shutdown MQTT
        // Extraction discards the IO channels
        if let ThingSyncState::Active(mut meta) = self {
            if let Some(hdlr) = meta.mqtt_handle {
                // Disconnect and Shutdown, generally disregarding
                // any errors in the closing process
                if let Err(e) = hdlr.disconnect() {
                    log::error!("Failed to disconnect handler: {}", e);
                }
                if let Err(e) = hdlr.shutdown() {
                    log::error!("Failed to shutdown MQTT: {}", e);
                }
            }

            // Explicitly discard the MQTT handle
            meta.mqtt_handle = None;

            for f in &[meta.ca_file_name, meta.cert_file_name, meta.key_file_name] {
                if let Some(ref file) = *f {
                    match fs::remove_file(file) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("failed to delete file \"{:?}\", error: {}", file, e);
                        }
                    }
                }
            }
        }
    }
}

/// `MetaThing` is the final state of the `ThingSyncState` state machine.
/// When a `HubThing` reaches this state, no further information is needed
/// to operate, however the MQTT connection may still need to be established
#[derive(Serialize, Deserialize)]
pub struct MetaThing {
    pub thing: Thing,
    pub resources: Vec<Resource>,

    pub ca_file_name: Option<PathBuf>,
    pub cert_file_name: Option<PathBuf>,
    pub key_file_name: Option<PathBuf>,

    #[serde(skip)] pub mqtt_handle: Option<MqttClient>,
}

impl MetaThing {
    /// Attempt to establish an MQTT connection for a device
    pub fn connect_mqtt(
        &mut self,
        mailbox: Sender<PartialThingMessage>,
        cert_storage: &PathBuf,
        mqtt_host: &str,
        mqtt_port: u16,
    ) -> Result<()> {
        let certs = self.thing
            .certs
            .as_ref()
            .ok_or_else(|| Error::from("Missing certificates!"))?;

        // AJM - This is not the best idea to write the certificates to a file.
        // Doing it for now, because the library (OpenSSL, depended on by
        // Rumqtt) only supports certificates from files, not strings.
        // TODO 2  - Also not necessary to write to file if it already exists
        let ca_file_name = cert_storage.join(&format!("{}.ca.crt", self.thing.id));
        let cert_file_name = cert_storage.join(&format!("{}.crt", self.thing.id));
        let key_file_name = cert_storage.join(&format!("{}.key", self.thing.id));

        for &(fname, body) in &[
            (&ca_file_name, &certs.ca),
            (&cert_file_name, &certs.cert),
            (&key_file_name, &certs.key),
        ] {
            // Scope to ensure file closed and written
            {
                let mut file = File::create(fname).chain_err(|| "Failed to create file!")?;
                file.write_all(body.as_bytes())
                    .chain_err(|| "Failed to write to file")?;
                file.sync_all().chain_err(|| "Failed to sync file")?;
            }
            fs::set_permissions(&fname, Permissions::from_mode(0o600))
                .expect("Failed to set certificate file permissions");
        }

        // Save file names
        self.ca_file_name = Some(ca_file_name.clone());
        self.cert_file_name = Some(cert_file_name.clone());
        self.key_file_name = Some(key_file_name.clone());

        let broker = format!("{}:{}", mqtt_host, mqtt_port);

        let opts = rumqtt::MqttOptions::new()
            .set_client_id(self.thing.id.hyphenated().to_string())
            .set_ca(ca_file_name)
            .set_client_cert(cert_file_name, key_file_name)
            .set_should_verify_ca(true)
            .set_broker(&broker)
            .set_keep_alive(5)
            .set_reconnect(10);

        // MQTT output channel must be ARC, in case multiple messages arrive at once
        let mqtt_channel = Arc::new(Mutex::new(mailbox));

        let msg_handler = rumqtt::MqttCallback::new().on_message(move |message| {
            let payload = String::from_utf8_lossy(message.payload.as_ref()).into_owned();
            log::info!(
                "Incoming MQTT message: {:?} payload: >>{}<<",
                message,
                payload
            );

            let sender = match mqtt_channel.lock() {
                Ok(tx) => tx,
                Err(e) => {
                    log::error!("Failed to lock mqtt sender: {}", e);
                    return;
                }
            };

            let rslt = sender.send(PartialThingMessage {
                topic: message.topic.to_string(),
                msg: payload,
            });

            if let Err(e) = rslt {
                log::error!("Failed to send MQTT message via channel: {}", e);
            }
        });

        let mut client =
            rumqtt::MqttClient::start(opts, Some(msg_handler)).chain_err(|| "Failed to connect!")?;

        let subscribes: Vec<(&str, rumqtt::QoS)> = self.resources
                    .iter()
                    .filter_map(|r| match r.method {
                        ResourceMethod::Sub => Some((r.uri.as_str(), rumqtt::QoS::Level0)), // TODO variable QoS?
                        ResourceMethod::Pub => None,
                    })
                    .collect();

        if subscribes.len() > 0 {
            client
                .subscribe(subscribes)
                .chain_err(|| "Failed to subscribe")?;
        }

        self.mqtt_handle = Some(client);

        Ok(())
    }

    pub fn process_messages(&mut self, msgs_from_hub: TryIter<PartialThingMessage>) -> Result<()> {
        // Messages from the cloud already are "pushed" to the final queue.
        // Messages from the hub need to be "pushed" to the cloud.
        if let Some(ref mut m_handle) = self.mqtt_handle {
            for msg in msgs_from_hub {
                log::info!("Sending: {:?}", msg);

                // TODO: https://jira.geeny.io/browse/DI-194
                use std::ascii::AsciiExt;
                let ascii_msg = msg.msg.chars().filter(|c| c.is_ascii()).collect::<String>();

                let tx_bytes = ascii_msg.into_bytes();

                m_handle
                    .publish(
                        &msg.topic,
                        rumqtt::QoS::Level0, // TODO configurable?
                        tx_bytes,
                    )
                    .chain_err(|| "Failed to publish")?;
            }
        }

        Ok(())
    }
}
