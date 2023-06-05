use std::net::SocketAddr;

use rocket::http::{ContentType, Status};
use rocket::{Data, State};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use serde_json::{from_str, json, to_string, Value};

use surrealdb::sql::{Id, Thing};
use tracing::info;

use crate::server::db::db_instance::DbInstance;
use crate::server::db::device_table::Device;
use crate::server::db::hash_table::DeviceHash;
use crate::server::db::Record;
use crate::server::utility::TextFieldExt;
use crate::server::utility::{self, gen_sha_256_hash};

#[get("/connect", data = "<data>")]
pub async fn connect(
    content_type: &ContentType,
    data: Data<'_>,
    remote_address: SocketAddr,
    db: &State<DbInstance>,
) -> Result<String, Status> {
    info!("Remote Address: {}", remote_address);
    // Process multipart form data
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

    // Return BadRequest(206) if there is an error parsing the request
    let multipart_form = match form_result {
        Ok(form) => form,
        Err(e) => return Err(Status::BadRequest),
    };

    // Extract the data from the form
    let device_id = multipart_form.texts.get("DeviceID");
    let os = multipart_form.texts.get("OS");
    let device_name = multipart_form.texts.get("DeviceName");
    let is_global = match multipart_form.texts.get("Global") {
        Some(_t) => true,
        None => false,
    };
    let location = multipart_form.texts.get("Location");
    let pin = multipart_form.texts.get("PIN");
    let read_only = match multipart_form.texts.get("ReadOnly") {
        Some(_t) => true,
        None => false,
    };

    let required_available = utility::verify_required_data(&[device_id, os, device_name, pin]);
    if !required_available {
        return Err(Status::BadRequest);
    }
    info!("All required Parameters available : {}", required_available);

    let device_id = device_id.first_text().unwrap();
    let os = os.first_text().unwrap();
    let device_name = device_name.first_text().unwrap();
    let location = location.first_text().unwrap();
    let pin = pin.first_text().unwrap();
    let database = &db.database;

    // Check for existing setup
    let device: Option<Device> = database.select(("device", &device_id)).await.unwrap();

    let device = match device {
        // Return Conflict if there is already a device with the same id
        Some(d) => return Err(Status::Conflict),
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
    let r: Option<Record> = database
        .create(("hash", &device_id))
        .content(&device_hash)
        .await
        .unwrap();
    Ok(device.uuid)
}

#[get("/database", data = "<data>")]
async fn sync_database(
    content_type: &ContentType,
    data: Data<'_>,
    remote_address: SocketAddr,
    db: &State<DbInstance>,
) -> Result<String, Status> {
    info!("Remote Address: {}", remote_address);
    // Process multipart form data
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("DeviceID"),
        MultipartFormDataField::text("DeviceName"),
        MultipartFormDataField::text("Global"),
        MultipartFormDataField::text("PIN"),
    ]);

    let form_result = MultipartFormData::parse(content_type, data, options).await;

    // Return BadRequest(206) if there is an error parsing the request
    let multipart_form = match form_result {
        Ok(form) => form,
        Err(e) => return Err(Status::BadRequest),
    };

    let device_id = multipart_form.texts.get("DeviceID");
    let device_name = multipart_form.texts.get("DeviceName");
    let is_global = match multipart_form.texts.get("Global") {
        Some(_t) => true,
        None => false,
    };
    let pin = multipart_form.texts.get("PIN");

    let device_id = device_id.first_text().unwrap();
    let device_name = device_name.first_text().unwrap();
    let pin = pin.first_text().unwrap();
    let database = &db.database;

    // Verify pin hash
    let database = &db.database;
    let hash: Result<Option<DeviceHash>, surrealdb::Error> =
        database.select(("hash", device_id)).await;
    let hash = match hash {
        Ok(d) => d,
        Err(e) => {
            error!("{e}");
            return Err(Status::InternalServerError);
        }
    };

    let hash = match hash {
        Some(d) => d,
        None => return Err(Status::BadRequest),
    };
    if hash.hash != pin {
        return Err(Status::Unauthorized);
    }

    Ok(String::from("rando"))
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

    // Return BadRequest(206) if there is an error parsing the request
    let multipart_form = match form_result {
        Ok(form) => form,
        Err(e) => return Err(Status::BadRequest),
    };

    let device_id = multipart_form.texts.get("DeviceID").first_text().unwrap();
    // let device_name = multipart_form.texts.get("DeviceName").first_text().unwrap();

    // Check if already present

    let result: Result<Option<Device>, surrealdb::Error> =
        database.select(("device", &device_id)).await;

    let result = match result {
        Err(e) => return Err(Status::InternalServerError),
        Ok(d) => d,
    };

    let result = match result {
        None => return Err(Status::Conflict),
        Some(d) => d,
    };

    return Ok(json!({
        "DeviceID": device_id,
        "DeviceName": result.name,
        "LastSync": result.last_sync,
        "Global": result.global
    }));

    // let device: Option<DeviceHash> = database
    //     .select(("hash", "Vivobook2210-34623"))
    //     .await
    //     .unwrap();

    // let device = match device {
    //     Some(d) => d,
    //     None => return (),
    // };
    // info!("{:?}", to_string(&device))
}
