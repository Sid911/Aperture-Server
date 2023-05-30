use std::net::SocketAddr;

use rocket::http::{ContentType, Status};
use rocket::{Data, State};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use serde_json::{from_str, to_string};
use surrealdb::Response;
use tracing::info;

use crate::server::db::db_instance::DbInstance;
use crate::server::db::{Device, Record};
use crate::server::utility;
use crate::server::utility::TextFieldExt;

#[get("/connect", data = "<data>")]
pub async fn connect(
    content_type: &ContentType,
    data: Data<'_>,
    remote_address: SocketAddr,
    db: &State<DbInstance>,
) -> Result<String, Status> {
    info!("Remote Address: {}", remote_address);
    // Process multipart form data
    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("DeviceID").size_limit(4096),
        MultipartFormDataField::text("OS"),
        MultipartFormDataField::text("DeviceName"),
        MultipartFormDataField::text("Global"),
        MultipartFormDataField::text("IP"),
        MultipartFormDataField::text("Location"),
        MultipartFormDataField::text("PIN"),
        MultipartFormDataField::text("ReadOnly"),
    ]);

    let form_result = MultipartFormData::parse(content_type, data, options).await;

    // Return BadRequest(206) if there is an error parsing the request
    let mut multipart_form = match form_result {
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
    let ip = multipart_form.texts.get("IP");
    let location = multipart_form.texts.get("Location");
    let pin = multipart_form.texts.get("PIN");
    let read_only = match multipart_form.texts.get("ReadOnly") {
        Some(_t) => true,
        None => false,
    };

    let required_available = utility::verify_required_data(&[device_id, os, device_name, ip, pin]);
    info!("All required Parameters available : {}", required_available);

    let device_id = device_id.first_text().unwrap();
    let os = os.first_text().unwrap();
    let device_name = device_name.first_text().unwrap();
    let location = location.first_text().unwrap();
    let pin = pin.first_text().unwrap();

    let database = &db.database;
    let device: Option<Device> = database.select(("device", &device_id)).await.unwrap();

    let device = match device {
        Some(d) => return Err(Status::Conflict),
        None => Device::new(
            device_id.clone(),
            device_name,
            is_global,
            read_only,
            from_str(&os).unwrap(),
        ),
    };

    let seralized = to_string(&device).unwrap();
    info!("Creating Device : {}", seralized);
    let r : Option<Record>= database
        .create(("device", device_id))
        .content(device)
        .await
        .unwrap();
    // info!("Device Created {:?}", r);

    Ok("Ok".to_string())
}

#[get("/database")]
async fn sync_database() {}

#[get("/server")]
async fn server_sync() {}
