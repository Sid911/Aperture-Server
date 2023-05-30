pub mod db_instance;
pub mod middleware;

use chrono::{self, Utc, DateTime};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OS {
    Android(f32),
    IOS(f32),
    Windows(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Device {
    device_id: String,
    last_sync: chrono::DateTime<Utc>,
    created_date: chrono::DateTime<Utc>,
    name: String,
    global: bool,
    read_only: bool,
    os: OS,
}

impl Device {
    pub fn new(id: String, name: String, global: bool, read_only:bool, os: OS) -> Device{
        Device{
            device_id: id,
            name,
            global,
            read_only,
            os,
            created_date: Utc::now(),
            last_sync: Utc::now(),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Record {
    #[allow(dead_code)]
    pub id: Thing,
}