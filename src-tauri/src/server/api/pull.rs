use rocket::{fs::NamedFile, http::ContentType, Data, State};
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};

use crate::{
    parse_multipart_form_texts,
    server::{
        db::{db_instance::DbInstance, local_table::LocalEntry},
        utility::{gen_sha_256_hash, TextFieldExt},
    },
};

use super::utility::{verify_device_id, verify_pin};

#[get("/file", data = "<data>")]
pub async fn pull_file(
    content_type: &ContentType,
    data: Data<'_>,
    db: &State<DbInstance>,
) -> Result<NamedFile, &'static str> {
    let options = MultipartFormDataOptions {
        max_data_bytes: 100 * 1024 * 1024,
        allowed_fields: vec![
            MultipartFormDataField::text("FileName"),
            MultipartFormDataField::text("RelativePath"),
            MultipartFormDataField::text("Global"),
            MultipartFormDataField::text("DeviceID"),
            MultipartFormDataField::text("DeviceName"),
            MultipartFormDataField::text("PIN"),
        ],
        ..MultipartFormDataOptions::default()
    };

    let form_result = MultipartFormData::parse(content_type, data, options).await;
    parse_multipart_form_texts!(
        multipart_form: form_result,
        parse_error: "Error: Could not parse the request";
        device_id: "DeviceID";
        _device_name: "DeviceName";
        file_name: "FileName";
        relative_path: "RelativePath";
        pin: "PIN";
    );

    let _is_global = match multipart_form.texts.get("Global") {
        Some(_t) => true,
        None => false,
    };

    let database = &db.database;

    // Check Device Entry
    let result = verify_device_id::<&'static str>(
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

    let local = match local {
        Some(l) => l,
        None => return Err("Could not find the file"),
    };

    let file_path = std::path::PathBuf::from(local.file_location);

    if file_path.exists() {
        let file_obj = NamedFile::open(file_path).await;
        match file_obj {
            Err(e) => {
                error!("{}", e);
                return Err("Error reading the file on expected path of the database");
            }
            Ok(f) => Ok(f),
        }
    } else {
        Err("file does not exist in the expected location")
    }
}

// #[get("/folder")]
// async fn pull_folder() {}
