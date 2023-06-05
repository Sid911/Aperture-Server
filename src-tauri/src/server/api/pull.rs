use rocket::{fs::NamedFile, http::ContentType, Data, State};
use rocket_multipart_form_data::{
    multer, MultipartFormData, MultipartFormDataError, MultipartFormDataField,
    MultipartFormDataOptions,
};

use crate::server::{
    db::{db_instance::DbInstance, device_table::Device, hash_table::DeviceHash, local_table::LocalEntry},
    utility::{gen_sha_256_hash, TextFieldExt},
};

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
    let device_id = multipart_form.texts.get("DeviceID");
    let device_name = multipart_form.texts.get("DeviceName");
    let file_name = multipart_form.texts.get("FileName");
    let realtive_path = multipart_form.texts.get("RelativePath");
    let is_global = match multipart_form.texts.get("Global") {
        Some(_t) => true,
        None => false,
    };
    let pin = multipart_form.texts.get("PIN");
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
        None => return Err("Could not find the file")
    };

    let file_path = std::path::PathBuf::from(local.file_location);

    if file_path.exists() {
        let file_obj = NamedFile::open(file_path).await;
        match file_obj {
            Err(e) =>{
                error!("{}",e);
                return Err("Error reading the file on expected path of the database");
            },
            Ok(f) => Ok(f)
        }
    } else {
        Err("file does not exist in the expected location")
    }
}

#[get("/folder")]
async fn pull_folder() {}
