use std::path::PathBuf;

use rocket::http::{ContentType, Status};
use rocket::{Data, State};
use rocket_include_static_resources::mime;
use rocket_multipart_form_data::{
    multer, MultipartFormData, MultipartFormDataError, MultipartFormDataField,
    MultipartFormDataOptions, RawField,
};
use rocket_raw_response::RawResponse;
use tokio::fs;
use tokio::task::spawn_blocking;
use tracing::info;

use crate::server::db::db_instance::DbInstance;
use crate::server::utility::TextFieldExt;

async fn save_image_to_documents(
    image: Option<Vec<RawField>>,
    file_name: String,
    relative_path: String,
    device_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(image) = image {
        let documents_dir = spawn_blocking(get_documents_dir).await?;
        let documents_dir = match documents_dir {
            Some(d) => d,
            None => return Ok(()),
        };
        let file_path = documents_dir
            .join("Aperture")
            .join(device_name)
            .join(relative_path)
            .join(file_name);

        // Modify the filename as desired
        info!("Saving Image: {:#?}", file_path.to_str());
        // Get the raw bytes of the image
        let image_data = image.into_iter().next().unwrap().raw;

        // Write the image data to the file
        fs::write(file_path, image_data).await?;
    }

    Ok(())
}

fn get_documents_dir() -> Option<PathBuf> {
    let documents_dir = dirs::document_dir();
    documents_dir
}

#[post("/file", data = "<data>")]
pub async fn push_file(
    content_type: &ContentType,
    data: Data<'_>,
    db: &State<DbInstance>,
) -> Result<Status, &'static str> {
    let options = MultipartFormDataOptions {
        max_data_bytes: 100 * 1024 * 1024,
        allowed_fields: vec![
            MultipartFormDataField::raw("image")
                .size_limit(100 * 1024 * 1024)
                .content_type_by_string(Some(mime::IMAGE_STAR))
                .unwrap(),
            MultipartFormDataField::file("img").size_limit(100 * 1024 * 1024),
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
    let img = multipart_form.files.get("img");

    let img = img.unwrap().first().unwrap();
    // Note: Might have to move file from tmp to the target path instead if I don't
    // Want to use raw file data
    // Which should be a good thing

    // Get Strings
    let device_id = device_id.first_text().unwrap();
    let device_name = device_name.first_text().unwrap();
    let pin = pin.first_text().unwrap();
    let file_name = file_name.first_text().unwrap();
    let relative_path = realtive_path.first_text().unwrap();

    let image = multipart_form.raw.remove("image");

    let img_status = save_image_to_documents(image, file_name, relative_path, device_name).await;
    if let Err(e) = img_status {
        error!("{}", e);
    }
    // let image = match image {
    //     Some(mut image) => {
    //         let raw = image.remove(0);

    //         let content_type = raw.content_type;
    //         let file_name = raw.file_name.unwrap_or_else(|| "Image".to_string());
    //         let data = raw.raw;

    //         RawResponse::from_vec(data, Some(file_name), content_type)
    //     }
    //     None => return Err("Please input a file."),
    // };
    Ok(Status::Accepted)
}

async fn push_folder() {}
