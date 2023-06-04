use std::net::SocketAddr;

use rocket::{
    http::{ContentType, Status},
    Data, State,
};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use serde_json::from_str;
use surrealdb::opt::PatchOp;

use crate::server::{
    db::{db_instance::DbInstance, device_table::Device, hash_table::DeviceHash, OS},
    utility::{self, TextFieldExt},
};

#[patch("/file")]
async fn modify_file() {}

#[post("/device", data = "<data>")]
pub async fn modfiy_device(
    db: &State<DbInstance>,
    data: Data<'_>,
    remote_address: SocketAddr,
    content_type: &ContentType,
) -> Result<Status, Status> {
    info!("Remote Address: {}", remote_address);
    // Process multipart form data
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("DeviceID"),
        MultipartFormDataField::text("OS"),
        MultipartFormDataField::text("DeviceName"),
        MultipartFormDataField::text("PIN"),
        MultipartFormDataField::text("RemoteAddr"),
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
    let pin = multipart_form.texts.get("PIN");
    let read_only = match multipart_form.texts.get("ReadOnly") {
        Some(_t) => true,
        None => false,
    };

    let required_available = utility::verify_required_data(&[device_id, pin]);
    info!("All required Parameters available : {}", required_available);
    if !required_available {
        return Err(Status::BadRequest);
    }

    let device_id = device_id.first_text().unwrap();
    let pin = pin.first_text().unwrap();
    let database = &db.database;

    // Verify Device ID
    let result: Result<Option<Device>, surrealdb::Error> =
        database.select(("device", &device_id)).await;

    let result = match result {
        Err(e) => return Err(Status::InternalServerError),
        Ok(d) => d,
    };

    let device = match result {
        None => return Err(Status::Conflict),
        Some(d) => d,
    };

    // Verify Pin
    let hash: Result<Option<DeviceHash>, surrealdb::Error> =
        database.select(("hash", &device_id)).await;
    let hash = match hash {
        Ok(d) => d,
        Err(e) => {
            error!("{e}");
            return Err(Status::InternalServerError);
        }
    };

    let device_hash = match hash {
        Some(d) => d,
        None => return Err(Status::BadRequest),
    };
    if device_hash.hash != pin {
        return Err(Status::Unauthorized);
    }

    // After verfied
    let os = os.first_text();
    if let Some(os_type) = os {
        let os: Result<OS, serde_json::Error> = from_str(&os_type);
        let os = match os {
            Ok(o) => o,
            Err(e) => {
                error!("Error Parsing OS : {}", e);
                return Err(Status::BadRequest);
            }
        };
        let _d: Device = database
            .update(("device", &device_id))
            .patch(PatchOp::replace("/os", os))
            .await
            .unwrap();
    }

    let device_name = device_name.first_text();
    if let Some(dev_name) = device_name {
        let _d: Device = database
            .update(("device", &device_id))
            .patch(PatchOp::replace("/name", dev_name))
            .await
            .unwrap();
    }

    let remote_addr = match multipart_form.texts.get("RemoteAddr") {
        Some(_t) => true,
        None => false,
    };

    if remote_addr {
        let _d: Device = database
            .update(("device", &device_id))
            .patch(PatchOp::replace(
                "/last_remote_addr",
                remote_address.to_string(),
            ))
            .await
            .unwrap();
    }

    Ok(Status::Ok)
}
