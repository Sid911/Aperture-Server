use std::net::SocketAddr;

use rocket::http::{ContentType, Status};
use rocket::{Data, State};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use serde_json::{from_str, json, to_string, Value};

use surrealdb::sql::{Id, Thing};
use tracing::info;

use crate::parse_multipart_form_texts;
use crate::server::api::utility::{verify_device_id, verify_pin};
use crate::server::db::db_instance::DbInstance;
use crate::server::db::device_table::Device;
use crate::server::db::hash_table::DeviceHash;
use crate::server::db::local_table::LocalEntry;
use crate::server::db::Record;
use crate::server::utility::TextFieldExt;

#[get("/connect", data = "<data>")]
pub async fn connect(
    content_type: &ContentType,
    data: Data<'_>,
    remote_address: SocketAddr,
    db: &State<DbInstance>,
) -> Result<String, Status> {
    info!("Remote Address: {}", remote_address);
    // Process multipart form data    let device_id:String;
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("DeviceID"),
        MultipartFormDataField::text("OS"),
        MultipartFormDataField::text("DeviceName"),
        MultipartFormDataField::text("Global"),
        MultipartFormDataField::text("Location"),
        MultipartFormDataField::text("PIN"),
        MultipartFormDataField::text("ReadOnly"),
    ]);

    let form_result = MultipartFormData::parse(content_type, data, options).await;

    //extract texts
    parse_multipart_form_texts!(
        multipart_form: form_result,
        // Return BadRequest(206) if there is an error parsing the request
        parse_error: Status::BadRequest;
        device_id: "DeviceID";
        os: "OS";
        device_name: "DeviceName";
        pin: "PIN";
    );

    // Extract the data from the form for bools
    let is_global = match multipart_form.texts.get("Global") {
        Some(_t) => true,
        None => false,
    };
    let read_only = match multipart_form.texts.get("ReadOnly") {
        Some(_t) => true,
        None => false,
    };

    let database = &db.database;

    // Check for existing setup
    let device: Option<Device> = database.select(("device", &device_id)).await.unwrap();

    let device = match device {
        // Return Conflict if there is already a device with the same id
        Some(_d) => return Err(Status::Conflict),
        None => Device::new(
            device_name,
            is_global,
            read_only,
            from_str(&os).unwrap(),
            remote_address.to_string(),
        ),
    };

    let seralized = to_string(&device).unwrap();
    info!("Creating Device : {:#?}", seralized);

    // let r: surrealdb::Response = database
    // .query(format!("CREATE device:{device_id} CONTENT {seralized}"))
    // .await.unwrap();
    let r: Option<Record> = database
        .create(("device", &device_id))
        .content(&device)
        .await
        .unwrap();
    info!("Device Created : {:#?}", r);

    // Create hash and store it using the pin
    let device_hash = DeviceHash::new(
        device.uuid.clone(),
        device.name,
        pin,
        Thing {
            tb: "device".to_string(),
            id: Id::from(&device_id),
        },
    );
    // let r = database
    // .query(format!("CREATE hash:{device_id} Content {}", to_string(&device_hash).unwrap()))
    // .await.unwrap();
    let _r: Option<Record> = database
        .create(("hash", &device_id))
        .content(&device_hash)
        .await
        .unwrap();
    Ok(device.uuid)
}

#[get("/database", data = "<data>")]
pub async fn sync_database(
    content_type: &ContentType,
    data: Data<'_>,
    remote_address: SocketAddr,
    db: &State<DbInstance>,
) -> Result<Value, &'static str> {
    info!("Remote Address: {}", remote_address);
    // Process multipart form data
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("DeviceID"),
        MultipartFormDataField::text("DeviceName"),
        MultipartFormDataField::text("Global"),
        MultipartFormDataField::text("PIN"),
    ]);

    let form_result = MultipartFormData::parse(content_type, data, options).await;

    parse_multipart_form_texts!(
        multipart_form: form_result,
        parse_error: "Error Parsing the request";
        device_id: "DeviceID";
        _device_name: "DeviceName";
        pin: "PIN";
    );

    let _is_global = match multipart_form.texts.get("Global") {
        Some(_t) => true,
        None => false,
    };

    let database = &db.database;
    // Check Device Entry
    let result = verify_device_id(
        database,
        &device_id,
        "Error: finding device in database",
        "Device is not present in database",
    )
    .await;

    if let Err(e) = result {
        return Err(e);
    }

    // Verify Pin
    let result = verify_pin(
        database,
        &device_id,
        &pin,
        "Error: finding device hash in database\nCould not verify",
        "Couldn't find any auth entires for device ID",
        "Unauthorized",
    )
    .await;

    if let Err(e) = result {
        return Err(e);
    }

    // Start sync logic

    let local_entries: Result<Vec<LocalEntry>, surrealdb::Error> =
        database.select(&device_id).await;
    let n = match local_entries {
        Ok(entires) => {
            let local_ids: Vec<Record> = database
                .select(&device_id)
                .await
                .expect("Error retriving ids");
            let e: Vec<LocalEntryWithId> = entires
                .into_iter()
                .enumerate()
                .map(|(index, entry)| LocalEntryWithId {
                    id: local_ids[index].id.id.to_string(),
                    entry,
                })
                .collect();
            e
        }
        Err(e) => {
            error!("Error retriving entires:  {}", e);
            return Err("Error: retriving entires");
        }
    };

    Ok(json!({
        "local_entries": n,
    }))
}

#[derive(serde::Serialize)]
struct LocalEntryWithId {
    id: String,
    entry: LocalEntry,
}

#[get("/server", data = "<data>")]
pub async fn server_sync(
    db: &State<DbInstance>,
    data: Data<'_>,
    content_type: &ContentType,
) -> Result<Value, Status> {
    let database = &db.database;

    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("DeviceID"),
        MultipartFormDataField::text("DeviceName"),
    ]);

    let form_result = MultipartFormData::parse(content_type, data, options).await;
    parse_multipart_form_texts!(
        multipart_form: form_result,
        parse_error: Status::BadRequest;
        device_id: "DeviceID";
    );

    // Check if already present
    let result = verify_device_id(
        database,
        &device_id,
        Status::InternalServerError,
        Status::Conflict,
    )
    .await;

    let device = match result {
        Err(e) => return Err(e),
        Ok(d) => d,
    };
    return Ok(json!({
        "DeviceID": device_id,
        "DeviceName": device.name,
        "LastSync": device.last_sync,
        "Global": device.global
    }));
}
