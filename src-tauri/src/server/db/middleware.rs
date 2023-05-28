use rocket::{
    fairing::{Fairing, Info, Kind, Result},
    serde::Deserialize,
    Build, Rocket,
};


use super::db_instance::DbInstance;


pub struct DbMiddleware;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct DbConfig {
    namespace: String,
    database: String,
    datastore: String,
}

#[rocket::async_trait]
impl Fairing for DbMiddleware {
    fn info(&self) -> Info {
        Info {
            name: "Database Middleware",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
        let figment = rocket.figment().clone();

        let db_config: DbConfig = figment.select("database").extract().unwrap();

        let db = match DbInstance::new_instance(
            db_config.namespace.clone(),
            db_config.database.clone(),
            db_config.datastore.clone(),
        )
        .await{
            Ok(db) => db,
            Err(e) => panic!("{}",e)
        };

        Ok(rocket.manage(db))
    }
}
