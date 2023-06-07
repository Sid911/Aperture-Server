use std::net::SocketAddr;

use rocket::{
    http::{ContentType, Status},
    Data, State,
};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};

use surrealdb::opt::PatchOp;

use crate::server::{
    api::utility::{verify_device_id, verify_pin},
    db::{db_instance::DbInstance, device_table::Device},
    utility::{self, TextFieldExt},
};

// #[patch("/file")]
// fn modify_file() {}

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
        Err(_e) => return Err(Status::BadRequest),
    };

    // Extract the data from the form
    let device_id = multipart_form.texts.get("DeviceID");
    // let os = multipart_form.texts.get("OS");
    let device_name = multipart_form.texts.get("DeviceName");
    let pin = multipart_form.texts.get("PIN");
    let _read_only = match multipart_form.texts.get("ReadOnly") {
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
    let result = verify_device_id::<Status>(
        database,
        &device_id,
        Status::InternalServerError,
        Status::Conflict,
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
        Status::InternalServerError,
        Status::BadRequest,
        Status::Unauthorized,
    )
    .await;

    if let Err(e) = result {
        return Err(e);
    }

    // After verfied
    // let os = os.first_text();
    // if let Some(os_type) = os {
    //     let os: Result<OS, serde_json::Error> = from_str(&os_type);
    //     let os = match os {
    //         Ok(o) => o,
    //         Err(e) => {
    //             error!("Error Parsing OS : {}", e);
    //             return Err(Status::BadRequest);
    //         }
    //     };
    //     let _d: Device = database
    //         .update(("device", &device_id))
    //         .patch(PatchOp::replace("/os", to_string(&os).unwrap()))
    //         .await
    //         .unwrap();
    // }

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
