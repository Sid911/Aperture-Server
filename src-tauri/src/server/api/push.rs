use rocket::{
    http::{ContentType, Status},
    Data, State,
};
use rocket_multipart_form_data::{
    multer, MultipartFormData, MultipartFormDataError, MultipartFormDataField,
    MultipartFormDataOptions,
};

use crate::server::{
    db::{
        db_instance::DbInstance, device_table::Device, hash_table::DeviceHash,
        local_table::LocalEntry,
    },
    utility::{gen_sha_256_hash, TextFieldExt},
};

use super::utility::{generate_blurhash, get_file_meta, is_image_file, save_file_to_documents};

#[post("/file", data = "<data>")]
pub async fn push_file(
    content_type: &ContentType,
    data: Data<'_>,
    db: &State<DbInstance>,
) -> Result<Status, &'static str> {
    let options = MultipartFormDataOptions {
        max_data_bytes: 100 * 1024 * 1024,
        allowed_fields: vec![
            MultipartFormDataField::file("File").size_limit(100 * 1024 * 1024),
            MultipartFormDataField::text("FileName"),
            MultipartFormDataField::text("RelativePath"),
            MultipartFormDataField::text("Global"),
            MultipartFormDataField::text("DeviceID"),
            MultipartFormDataField::text("DeviceName"),
            MultipartFormDataField::text("PIN"),
        ],
        ..MultipartFormDataOptions::default()
    };

    let mut multipart_form = match MultipartFormData::parse(content_type, data, options).await {
        Ok(multipart_form_data) => multipart_form_data,
        Err(err) => {
            match err {
                MultipartFormDataError::DataTooLargeError(_) => {
                    return Err("The file is too large.");
                }
                MultipartFormDataError::DataTypeError(_) => {
                    return Err("The file is not an image.");
                }
                MultipartFormDataError::MulterError(multer::Error::IncompleteFieldData {
                    ..
                })
                | MultipartFormDataError::MulterError(multer::Error::IncompleteHeaders {
                    ..
                }) => {
                    // may happen when we set the max_data_bytes limitation
                    return Err("The request body seems too large.");
                }
                _ => panic!("{:?}", err),
            }
        }
    };
    // Get fields
    let device_id = multipart_form.texts.get("DeviceID");
    let device_name = multipart_form.texts.get("DeviceName");
    let file_name = multipart_form.texts.get("FileName");
    let realtive_path = multipart_form.texts.get("RelativePath");
    let is_global = match multipart_form.texts.get("Global") {
        Some(_t) => true,
        None => false,
    };
    let pin = multipart_form.texts.get("PIN");
    let file = multipart_form.files.get("File");

    let file = file.unwrap().first().unwrap();

    // Get Strings
    let device_id = device_id.first_text().unwrap();
    let device_name = device_name.first_text().unwrap();
    let pin = pin.first_text().unwrap();
    let file_name = file_name.first_text().unwrap();
    let relative_path = realtive_path.first_text().unwrap();

    // Check Device Entry
    let database = &db.database;
    let result: Result<Option<Device>, surrealdb::Error> =
        database.select(("device", &device_id)).await;

    let result = match result {
        Err(e) => return Err("Error: finding device in database"),
        Ok(d) => d,
    };

    let device = match result {
        None => return Err("Device is not present in database"),
        Some(d) => d,
    };

    // Verify Pin
    let hash: Result<Option<DeviceHash>, surrealdb::Error> =
        database.select(("hash", &device_id)).await;
    let hash = match hash {
        Ok(d) => d,
        Err(e) => {
            error!("{e}");
            return Err("Error: finding device hash in database\nCould not verify");
        }
    };

    let device_hash = match hash {
        Some(d) => d,
        None => return Err("Couldn't find any auth entires for device ID"),
    };
    if device_hash.hash != gen_sha_256_hash(&pin) {
        return Err("Unauthorized");
    }

    // Save File
    let file_status = save_file_to_documents(file, &file_name, &relative_path, &device_name).await;
    let file_path = match file_status {
        Some(f) => f,
        None => {
            return Err("Unable to save file on server");
        }
    };

    let file_meta = match get_file_meta(&file_path) {
        Some(f) => f,
        None => return Err("Error: Unable to get file metadata "),
    };
    let check_image = is_image_file(&file_path.clone());

    // Check for local Entry
    let file_id = gen_sha_256_hash(&(relative_path + &file_name));
    let local: Result<Option<LocalEntry>, surrealdb::Error> =
        database.select((&device_id, &file_id)).await;

    let local = match local {
        Ok(d) => d,
        Err(e) => {
            error!("{e}");
            return Err("Error: finding device hash in database\nCould not verify");
        }
    };
    let new_local_entry = LocalEntry::new(
        device_id.clone(),
        file_name.clone(),
        file_meta.len(),
        String::from(file_path.to_str().unwrap()),
        file.content_type.clone(),
        match check_image {
            true => generate_blurhash(&file_path).await,
            false => None,
        },
    );
    let _res = match local {
        Some(_instance) => {
            let local_l: Result<LocalEntry, surrealdb::Error> = database
                .update((&device_id, &file_id))
                .content(new_local_entry)
                .await;
            local_l
        }
        None => {
            let local_l: Result<LocalEntry, surrealdb::Error> = database
                .create((&device_id, &file_id))
                .content(new_local_entry)
                .await;
            local_l
        }
    };

    Ok(Status::Accepted)
}

async fn push_folder() {}
