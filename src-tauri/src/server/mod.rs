mod api;
pub mod db;
mod utility;

use db::middleware::DbMiddleware;
use rocket::{
    figment::{
        providers::{Format, Toml},
        Figment,
    },
    request::Request,
    response::content::RawHtml,
    Build, Config,
};

static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
    "/favicon-16.png" => favicon_png => "favicon-png",
}

/*

API for Aperture server starts here.
 */

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Oh no! We couldn't find the requested path '{}'", req.uri())
}

#[get("/")]
fn index() -> RawHtml<&'static str> {
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
        .register("/", catchers![not_found])
        .mount("/", routes![favicon, favicon_png])
        .mount("/", routes![index])
        .mount(
            "/sync",
            routes![
                api::sync::connect,
                api::sync::server_sync,
                api::sync::sync_database
            ],
        )
        .mount("/pull", routes![api::pull::pull_file])
        .mount("/push", routes![api::push::push_file])
        .mount("/modify", routes![api::modify::modfiy_device]);
    return build;
}
