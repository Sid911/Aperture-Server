pub mod db_instance;
pub mod device_table;
pub mod hash_table;
pub mod local_table;
pub mod middleware;


use surrealdb::sql::Thing;

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

/*  These are all persistant settings, these are immutable
   for rollback etc.
*/
// pub struct Server {
//     global_pin: String,
//     local_pin: String,
// }

// pub enum Store {
//     Device(local_table::LocalEntry),
//     Global(),
// }
