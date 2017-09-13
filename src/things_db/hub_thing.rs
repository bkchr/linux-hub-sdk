use std::sync::mpsc::{channel, Receiver, Sender};

use log;
use uuid::Uuid;

use errors::*;
use things_db::PartialThingMessage;
use things_db::state::ThingSyncState;
use things_db::runner::CarePackage;


#[derive(Serialize, Deserialize)]
pub struct HubThing {
    pub thing: ThingSyncState,

    #[serde(skip)]
    pub modem: HubModem,
}

impl HubThing {
    pub fn new(thing: ThingSyncState) -> Self {
        Self {
            thing: thing,
            modem: HubModem::default(),
        }
    }

    pub fn manage(&mut self, package: &CarePackage) -> Result<Option<Uuid>> {
        use self::ThingSyncState::*;

        let mut retval = None;

        let new_state = match (&mut self.thing, package.token_opt.as_ref()) {
            // A device has been created, and we have a valid token
            (&mut Created(ref req), Some(token)) => {
                ThingSyncState::create_new_thing(&package.config.api, token, req)
            }

            // A device needs metadata, and we have a valid token
            (&mut GatheringMetadata(ref thing), Some(token)) => {
                ThingSyncState::gather_thing_metadata(&package.config.api, token, thing)
            }

            // A device has metadata, but needs an MQTT connection
            (&mut Active(ref mut active), _) if active.mqtt_handle.is_none() => {
                active.connect_mqtt(
                    self.modem.cloud_to_hub_sender.clone(),
                    &package.config.certificate_storage,
                    &package.config.mqtt_host,
                    package.config.mqtt_port,
                )?;

                None
            }

            // A device is doing business
            (&mut Active(ref mut active), _) => {
                let packet_iter = self.modem.hub_to_cloud_receiver.try_iter();

                if let Err(e) = active.process_messages(packet_iter) {
                    log::error!("Error: {}", e);
                }
                None
            }
            _ => None,
        };

        // A transition occurred
        if let Some(state) = new_state {
            log::info!("Transition from {} to {}", self.thing, state);

            self.thing = state;

            if let Active(ref thing) = self.thing {
                retval = Some(thing.thing.id);
            }
        }
        Ok(retval)
    }

    pub fn extract(self) -> ThingSyncState {
        self.thing
    }
}

pub struct HubModem {
    pub cloud_to_hub_sender: Sender<PartialThingMessage>,
    pub cloud_to_hub_receiver: Receiver<PartialThingMessage>,
    pub hub_to_cloud_sender: Sender<PartialThingMessage>,
    pub hub_to_cloud_receiver: Receiver<PartialThingMessage>,
}

impl Default for HubModem {
    fn default() -> Self {
        let (cth_tx, cth_rx) = channel();
        let (htc_tx, htc_rx) = channel();
        Self {
            cloud_to_hub_sender: cth_tx,
            cloud_to_hub_receiver: cth_rx,
            hub_to_cloud_sender: htc_tx,
            hub_to_cloud_receiver: htc_rx,
        }
    }
}
