use std::net::SocketAddr;

use rocket::http::{ContentType, Status};
use rocket::Data;
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};

use crate::server::utility;

#[get("/connect", data = "<data>")]
pub async fn connect(
    content_type: &ContentType,
    data: Data<'_>,
    remote_address: SocketAddr,
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

    Ok("Got it".to_string())
}

#[get("/database")]
async fn sync_database() {}


#[get("/server")]
async fn server_sync() {}