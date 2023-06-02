use chrono::Utc;
use uuid::Uuid;

use super::OS;

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
