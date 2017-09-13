use std::collections::HashMap;
use std::sync::mpsc::Sender;

use log;
use uuid::Uuid;
use geeny_api::models::ThingRequest;

use errors::*;
use things_db::PartialThingMessage;
use things_db::state::ThingSyncState;
use things_db::runner::CarePackage;
use things_db::hub_thing::HubThing;

#[derive(Serialize, Deserialize, Default)]
pub struct ThingDb {
    // primary is serial number
    primary: HashMap<String, HubThing>,

    // secondary is Geeny id
    secondary: HashMap<Uuid, String>,
}

// Internal data structure-y things
impl ThingDb {
    fn insert_primary(&mut self, pkey: String, data: HubThing) {
        if let Some(old_data) = self.primary.insert(pkey, data) {
            log::warn!("Unexpected primary insert, replacing {}", old_data.thing)
        }
    }

    fn insert_secondary(&mut self, pkey: String, skey: Uuid) -> Result<()> {
        if self.primary.contains_key(&pkey) {
            if let Some(old_pkey) = self.secondary.insert(skey, pkey) {
                log::warn!("Unexpected secondary insert, replacing {}", old_pkey);
            }
            Ok(())
        } else {
            Err(Error::from(
                format!("No matching Serial Number for this UUID: {}", skey),
            ))
        }
    }

    fn remove_by_primary(&mut self, pkey: &str) -> Option<HubThing> {
        let retval = match self.primary.remove(pkey) {
            Some(val) => val,
            _ => return None,
        };

        // hm.
        let needle = self.secondary
            .iter()
            .find(|&(_, p)| p == pkey)
            .map(|(s, _)| *s);

        if let Some(s) = needle {
            let _ = self.secondary.remove(&s);
        }

        Some(retval)
    }

    fn receive_from_cloud(&self, pkey: &str) -> Result<Vec<PartialThingMessage>> {
        if let Some(m) = self.primary.get(pkey) {
            Ok(m.modem.cloud_to_hub_receiver.try_iter().collect())
        } else {
            bail!(
                "Tried to receive from cloud for s/n {}, does not exist",
                pkey
            )
        }
    }

    fn sender_to_cloud(&self, pkey: &str) -> Result<Sender<PartialThingMessage>> {
        if let Some(m) = self.primary.get(pkey) {
            Ok(m.modem.hub_to_cloud_sender.clone())
        } else {
            bail!(
                "Tried to get sender from hub for s/n {}, does not exist",
                pkey
            )
        }
    }

    fn contains_primary(&self, pkey: &str) -> bool {
        self.primary.contains_key(pkey)
    }
}

// Public interface
impl ThingDb {
    pub fn manage(&mut self, package: CarePackage) {
        let mut new_uuid_pairs = vec![];

        for (serial, doppel) in &mut self.primary {
            match doppel.manage(&package) {
                Err(e) => log::error!("Error in mgmt: {}", e),
                Ok(Some(uuid)) => {
                    new_uuid_pairs.push((uuid, serial.clone()));
                }
                Ok(None) => {}
            }
        }

        for (uuid, serial) in new_uuid_pairs.drain(..) {
            if let Err(e) = self.insert_secondary(serial, uuid) {
                log::error!("{}", e);
            }
        }
    }

    pub fn unpair_all(&mut self) {
        self.secondary.clear();

        for (_, doppel) in self.primary.drain() {
            doppel.extract().consume();
        }
    }

    pub fn unpair(&mut self, serial_number: &str) -> Result<()> {
        let to_delete = self.remove_by_primary(serial_number)
            .ok_or_else(|| Error::from("No device with that serial number found."))?;

        to_delete.extract().consume();

        Ok(())
    }

    pub fn add_thing(&mut self, new_thing: ThingRequest) -> Result<()> {
        if self.contains_primary(&new_thing.serial_number) {
            bail!("Duplicate device!")
        }

        self.insert_primary(
            new_thing.serial_number.clone(),
            HubThing::new(ThingSyncState::Created(new_thing)),
        );
        Ok(())
    }

    pub fn contains_serial(&self, serial_number: &str) -> bool {
        self.contains_primary(serial_number)
    }

    pub fn hub_tx(&mut self, serial_number: &str, msgs: &[PartialThingMessage]) -> Result<()> {
        let cloud_sender = self.sender_to_cloud(serial_number)?;

        for msg in msgs {
            cloud_sender
                .send(msg.clone())
                .chain_err(|| "Failed to send! Other side hung up?")?;
        }
        Ok(())
    }

    pub fn hub_rx(&mut self, serial_number: &str) -> Result<Vec<PartialThingMessage>> {
        self.receive_from_cloud(serial_number)
    }
}
