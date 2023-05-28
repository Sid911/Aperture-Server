use rocket::http::ContentType;
use rocket::Data;
use rocket_include_static_resources::mime;
use rocket_multipart_form_data::{MultipartFormDataOptions, MultipartFormDataField, MultipartFormDataError, MultipartFormData, multer};
use rocket_raw_response::RawResponse;




#[post("/file/<device_id>", data = "<data>")]
pub async fn push_file(
    device_id: String,
    content_type: &ContentType,
    data: Data<'_>,
) -> Result<RawResponse, &'static str> {
    let options = MultipartFormDataOptions {
        max_data_bytes: 33 * 1024 * 1024,
        allowed_fields: vec![MultipartFormDataField::raw("image")
            .size_limit(32 * 1024 * 1024)
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .unwrap()],
        ..MultipartFormDataOptions::default()
    };

    let mut multipart_form_data = match MultipartFormData::parse(content_type, data, options).await
    {
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

    let image = multipart_form_data.raw.remove("image");

    match image {
        Some(mut image) => {
            let raw = image.remove(0);

            let content_type = raw.content_type;
            let file_name = raw.file_name.unwrap_or_else(|| "Image".to_string());
            let data = raw.raw;

            Ok(RawResponse::from_vec(data, Some(file_name), content_type))
        }
        None => Err("Please input a file."),
    }
}


async fn push_folder() {}