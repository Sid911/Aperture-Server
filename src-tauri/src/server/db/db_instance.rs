use surrealdb::{Surreal, engine};

pub struct DbInstance{
    #[cfg(release)]
    database: Surreal<engine::local::Db>,
    #[cfg(not(release))]
    database: Surreal<engine::remote::ws::Client>
}

impl DbInstance {
    #[cfg(release)]
    pub async fn new_instance(namespace: String, database: String, datastore: String) -> Result<Self,surrealdb::Error> {
        let ds = Surreal::new::<engine::local::File>(datastore.as_str()).await?;
    
        // Select a specific namespace / database
        ds.use_ns(namespace).use_db(database).await?;
        Ok(Self {
            database: ds
        })
    }
    #[cfg(not(release))]
    pub async fn new_instance(namespace: String, database: String, datastore: String) -> Result<Self,surrealdb::Error> {
        use surrealdb::opt::auth::Root;

        let ds = Surreal::new::<engine::remote::ws::Ws>("localhost:8888").await?;
        ds.signin(Root{
            username: "root",
            password: "root"
        });
        // Select a specific namespace / database
        ds.use_ns(namespace).use_db(database).await?;
        Ok(Self {
            database: ds
        })
    }


}