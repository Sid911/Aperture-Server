mod db;

use rocket::{request::Request, figment::{Figment, providers::{Toml, Format}}, Config, http::{ContentType, Status}, response::content::RawHtml, Build, Response};
use db::middleware::{ DbMiddleware};
use rocket_include_static_resources::mime;
use rocket_multipart_form_data::{MultipartFormDataField, MultipartFormDataOptions, MultipartFormData, MultipartFormDataError, multer, Repetition};
use rocket::Data;
use rocket_raw_response::RawResponse;
use tracing::info;

static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
    "/favicon-16.png" => favicon_png => "favicon-png",
}



/*

API for Aperture server starts here.
 */
#[get("/connect", data = "<data>")]
async fn connect(content_type: &ContentType, data: Data<'_>) -> Result<String, Status>{
    let mut options = MultipartFormDataOptions::with_multipart_form_data_fields(
        vec! [
            MultipartFormDataField::text("DeviceID").size_limit(4096),
            MultipartFormDataField::text("OS"),
            MultipartFormDataField::text("DeviceName"),
            MultipartFormDataField::text("email"),
            MultipartFormDataField::text("global")
        ]
    );
    let mut multipart_form_data = MultipartFormData::parse(content_type, data, options).await;


    match multipart_form_data{
        Ok(form) => info!("Form is good"),
        Err(e) => return Err(Status::BadRequest),
    };

    Ok("Got it".to_string())
}


#[post("/push_file/<device_id>", data = "<data>")]
async fn push_file(device_id: String, content_type: &ContentType, data: Data<'_>) -> Result<RawResponse, &'static str> {
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
                },
                MultipartFormDataError::DataTypeError(_) => {
                    return Err("The file is not an image.");
                },
                MultipartFormDataError::MulterError(multer::Error::IncompleteFieldData {
                    ..
                })
                | MultipartFormDataError::MulterError(multer::Error::IncompleteHeaders {
                    ..
                }) => {
                    // may happen when we set the max_data_bytes limitation
                    return Err("The request body seems too large.");
                },
                _ => panic!("{:?}", err),
            }
        },
    };

    let image = multipart_form_data.raw.remove("image");

    match image {
        Some(mut image) => {
            let raw = image.remove(0);

            let content_type = raw.content_type;
            let file_name = raw.file_name.unwrap_or_else(|| "Image".to_string());
            let data = raw.raw;

            Ok(RawResponse::from_vec(data, Some(file_name), content_type))
        },
        None => Err("Please input a file."),
    }
}

async fn push_folder() {
    
}


async fn sync_database(){

}

async fn modify_file(){

}


async fn pull_file(){

}

async fn pull_folder(){

}

async fn server_sync(){

}



#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Oh no! We couldn't find the requested path '{}'", req.uri())
}

#[get("/")]
fn index() -> RawHtml<&'static str>{
    return RawHtml("<html><head><title> hello </title></head><body>Jello</body></html>");
    
}
pub fn rocket() -> rocket::Rocket<Build> {
    let figment = Figment::from(Config::default())
    .merge(Toml::file("Rocket.toml").nested())
    .merge(Toml::file("App.toml").nested());

    let build = rocket::custom(figment)
    .attach(DbMiddleware)
    .attach(static_resources_initializer!(
        "favicon" => "assets/favicon.ico",
        "favicon-png" => "assets/favicon-32x32.png",
    ))
    .register("/",catchers![not_found])
    .mount("/", routes![favicon, favicon_png])
    .mount("/", routes![index,connect])
    .mount("/", routes![push_file]);
    return build;
}