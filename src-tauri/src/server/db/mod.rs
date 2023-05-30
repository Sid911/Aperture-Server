pub mod db_instance;
pub mod middleware;

use chrono::{self, Utc};
use sha2::{Digest, Sha256};
use surrealdb::sql::{Algorithm, Thing, Uuid};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OS {
    Android(f32),
    IOS(f32),
    Windows(String),
}

#[derive(Debug, serde::Deserialize)]
pub struct Record {
    #[allow(dead_code)]
    pub id: Thing,
}

/*  Device Struct is for db integration
 */
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Device {
    pub uuid: Uuid,
    pub last_sync: chrono::DateTime<Utc>,
    pub created_date: chrono::DateTime<Utc>,
    pub name: String,
    pub global: bool,
    pub read_only: bool,
    pub os: OS,
}

impl Device {
    pub fn new(name: String, global: bool, read_only: bool, os: OS) -> Device {
        Device {
            uuid: Uuid::new_v4(),
            name,
            global,
            read_only,
            os,
            created_date: Utc::now(),
            last_sync: Utc::now(),
        }
    }
}

// Device Hash for specific devices
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceHash {
    uuid: Uuid,
    device_name: String,
    hash: String,
    hash_type: Algorithm,
}

impl DeviceHash {
    pub fn new(uuid: Uuid, device_name: String, pin: String) -> DeviceHash {
        let mut hasher = Sha256::new();
        hasher.update(b"Hello, world!");

        let result = hasher.finalize();

        // Convert the hash to a string.
        let hash_string = format!("{:x}", result);

        DeviceHash {
            uuid,
            device_name,
            hash: hash_string,
            hash_type: Algorithm::Hs256,
        }
    }
}

/*  These are all persistant settings, these are immutable
   for rollback etc.
*/
pub struct Server {
    global_pin: String,
    local_pin: String,
}
