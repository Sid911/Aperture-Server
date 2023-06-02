use surrealdb::sql::{Algorithm, Thing};
use uuid::Uuid;

use crate::server::utility::gen_sha_256_hash;

// Device Hash for specific devices
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceHash {
    uuid: Uuid,
    device: Thing,
    device_name: String,
    pub hash: String,
    hash_type: Algorithm,
}

impl DeviceHash {
    pub fn new(uuid: Uuid, device_name: String, pin: String, obj: Thing) -> DeviceHash {
        // Convert the hash to a string.
        let hash_string = gen_sha_256_hash(&pin);

        DeviceHash {
            uuid,
            device: obj,
            device_name,
            hash: hash_string,
            hash_type: Algorithm::Hs256,
        }
    }
}
