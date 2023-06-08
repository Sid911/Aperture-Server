use rocket::log::private::info;
use rocket_multipart_form_data::FileField;
use surrealdb::{engine::remote::ws::Client, Surreal};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::server::{db::{device_table::Device, hash_table::DeviceHash}, utility::gen_sha_256_hash};

// Mulitpart form extraction

    /// Extracts multipart-form texts by taking the parsed result and keys
    /// for the field
    /// # Example
    /// ```
    /// let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
    ///     MultipartFormDataField::text("DeviceID"),
    ///     MultipartFormDataField::text("OS"),
    ///     MultipartFormDataField::text("DeviceName"),
    ///     MultipartFormDataField::text("Global"),
    ///     MultipartFormDataField::text("Location"),
    ///     MultipartFormDataField::text("PIN"),
    ///     MultipartFormDataField::text("ReadOnly"),
    /// ]);
    /// 
    /// let form_result = MultipartFormData::parse(content_type, data, options).await;
    /// 
    /// 
    /// parse_multipart_form_texts!(
    ///     multipart_form: form_result,
    ///     // Return BadRequest(206) if there is an error parsing the request
    ///     parse_error: Status::BadRequest;
    ///     device_id: "DeviceID";
    ///     os: "OS";
    ///     device_name: "DeviceName";
    ///     pin: "PIN";
    /// );
    /// ```
    ///
#[macro_export]
macro_rules! parse_multipart_form_texts {
    () => {};
    ($multipart:ident: $form:expr, parse_error: $error_return:expr; $($field:ident: $elem:expr;)*) => {
        let $multipart = match $form {
            Ok(form) => form,
            Err(_e) => return Err($error_return),
        };
        
        $(
            let $field = $multipart.texts
            .get($elem)
            .first_text()
            .expect(format!("{} not found", $elem).as_str());
        )*
    };
    // let multipart_form = match MultipartFormData::parse(content_type, data, options).await {
    //     Ok(multipart_form_data) => multipart_form_data,
    //     Err(err) => {
    //         match err {
    //             MultipartFormDataError::DataTooLargeError(_) => {
    //                 return Err("The file is too large.");
    //             }
    //             MultipartFormDataError::DataTypeError(_) => {
    //                 return Err("The file is not an image.");
    //             }
    //             MultipartFormDataError::MulterError(multer::Error::IncompleteFieldData {
    //                 ..
    //             })
    //             | MultipartFormDataError::MulterError(multer::Error::IncompleteHeaders {
    //                 ..
    //             }) => {
    //                 // may happen when we set the max_data_bytes limitation
    //                 return Err("The request body seems too large.");
    //             }
    //             _ => panic!("{:?}", err),
    //         }
    //     }
    // };
}


// File based Utility functions
pub fn get_file_meta(path: &std::path::PathBuf) -> Option<std::fs::Metadata> {
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.is_file() {
            return Some(metadata);
        }
    }
    None
}

pub async fn save_file_to_documents(
    file: &FileField,
    file_name: &str,
    relative_path: &str,
    device_id: &str,
) -> Option<std::path::PathBuf> {
    let documents_dir = match get_documents_dir().await {
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to get documents directory: {}", e);
            return None;
        }
    };

    let device_dir = documents_dir.join("Aperture").join(device_id);
    let target_dir = device_dir.join(relative_path);

    if let Err(e) = tokio::fs::create_dir_all(&target_dir).await {
        error!("Failed to create target directory: {}", e);
        return None;
    }

    let target_file_path = target_dir.join(file_name);
    let mut target_file = match tokio::fs::File::create(&target_file_path).await {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to create target file: {}", e);
            return None;
        }
    };

    let file_path = &file.path;
    let mut reader = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to open source file: {}", e);
            return None;
        }
    };
    let mut buffer = vec![0; 4096];
    loop {
        match reader.read(&mut buffer).await {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                if let Err(e) = target_file.write_all(&buffer[..bytes_read]).await {
                    error!("Failed to write to target file: {}", e);
                    return None;
                }
            }
            Err(e) => {
                error!("Failed to read from source file: {}", e);
                return None;
            }
        }
    }

    Some(target_file_path.clone())
}

pub fn is_image_file(file_path: &std::path::Path) -> bool {
    info!("is_image_file : {:?}", file_path);
    let file_extension = file_path
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_lowercase();
    let image_extensions = vec!["jpg", "jpeg", "png", " "]; // Add more image extensions if needed
    image_extensions.contains(&file_extension.as_str())
}

pub async fn generate_blurhash(file_path: &std::path::Path) -> Option<String> {
    let mut file = tokio::fs::File::open(file_path).await.unwrap();
    let mut image_data = Vec::new();
    file.read_to_end(&mut image_data).await.unwrap();

    let width = 32;
    let height = 32;
    let components_x = 4;
    let components_y = 3;

    return Some(blurhash::encode(
        components_x,
        components_y,
        width,
        height,
        &image_data,
    ));
}

pub async fn get_documents_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    match dirs::document_dir() {
        Some(documents_dir) => Ok(documents_dir),
        None => Err("Failed to get documents directory".into()),
    }
}

// Handle database checks

pub async fn verify_device_id<T>(
    database: &Surreal<Client>,
    device_id: &String,
    surreal_error: T,
    not_found_error: T,
) -> Result<Device, T> {
    let result: Result<Option<Device>, surrealdb::Error> =
        database.select(("device", device_id)).await;

    let result = match result {
        Err(_e) => return Err(surreal_error),
        Ok(d) => d,
    };

    let device = match result {
        None => return Err(not_found_error),
        Some(d) => d,
    };
    return Ok(device);
}

pub async fn verify_pin<T>(
    database: &Surreal<Client>,
    device_id: &String,
    pin: &String,
    surreal_error: T,
    not_found_error: T,
    incorrect_pin_error: T,
) -> Result<DeviceHash, T>{
    let hash: Result<Option<DeviceHash>, surrealdb::Error> =
    database.select(("hash", device_id)).await;
let hash = match hash {
    Ok(d) => d,
    Err(e) => {
        error!("{e}");
        return Err(surreal_error);
    }
};

let device_hash = match hash {
    Some(d) => d,
    None => return Err(not_found_error),
};
if device_hash.hash != gen_sha_256_hash(pin) {
    return Err(incorrect_pin_error);
}
return Ok(device_hash);
}