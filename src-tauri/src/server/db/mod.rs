pub mod middleware;

use rocket::{
    fairing::{Fairing, Info, Kind, Result},
    serde::Deserialize,
    Build, Rocket,
};
use surrealdb::{dbs::Session, kvs::Datastore, sql::Value, Error};

pub struct Db {
    session: Session,
    datastore: Datastore,
}

impl Db {
    pub async fn new(namespace: &str, database: &str, datastore: &str) -> Self {
        Self {
            session: Session::for_db(namespace.to_string(), database.to_string()),
            datastore: Datastore::new(&datastore).await.unwrap(),
        }
    }

    pub async fn query(&self, statement: &str) -> Result<Vec<Value>, Error> {
        let responses = self
            .datastore
            .execute(statement, &self.session, None, false)
            .await?;

        let mut results = Vec::new();

        for response in responses {
            results.push(response.result?.first());
        }

        Ok(results)
    }
}

pub struct DbFairing;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct DbConfig {
    namespace: String,
    database: String,
    datastore: String,
}

#[rocket::async_trait]
impl Fairing for DbFairing {
    fn info(&self) -> Info {
        Info {
            name: "Database",
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
        let figment = rocket.figment().clone();

        let db_config: DbConfig = figment.select("database").extract().unwrap();

        let db = Db::new(
            &db_config.namespace,
            &db_config.database,
            &db_config.datastore,
        )
        .await;

        Ok(rocket.manage(db))
    }
}
