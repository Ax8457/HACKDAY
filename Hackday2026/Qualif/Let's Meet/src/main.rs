/*
	Imports
*/
#[macro_use] extern crate rocket;
mod auth;
mod structs;
mod routes;
mod utilities;
mod api;
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::http::Status;
use mongodb::Client;
use rocket::figment::Figment;

/*
	Security guard => HMAC session token for Authenticated users 
*/
pub struct AuthenticatedUser {pub username: String,pub referer: String, }

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let referer_value = request.headers().get_one("Referer")
            .unwrap_or("direct")
            .to_string();
        if let Some(c) = request.cookies().get("session_token") {
            if let Some((json_str, tag)) = auth::decode_token(c.value()) {
                if auth::check_token(auth::key, json_str.as_bytes(), &tag) {
                    if let Ok(p) = serde_json::from_str::<structs::TokenPayload>(&json_str) {
                        return Outcome::Success(AuthenticatedUser { 
                            username: p.username,
                            referer: referer_value 
                        });
                    }
                }
            }
        }
        Outcome::Forward(Status::Unauthorized)
    }
}

/*
	Launch server, attach routes, static style files, db
*/

#[launch]
async fn rocket() -> _ {
    let config = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", 8000));

    let mongo = Client::with_uri_str("mongodb://127.0.0.1:27017")
        .await
        .expect("Failed to connect");
    
    let mongo_api = mongo.clone();
    tokio::spawn(async move {
        let _ = api::launch_api(mongo_api).await;
    });

    rocket::custom(config)
        .manage(mongo)
        .attach(rocket_dyn_templates::Template::fairing())
        .mount("/", routes![
            routes::choosemonth,
            routes::index, routes::index_error,
            routes::account, routes::account_error, routes::add_member, routes::leave_route, routes::change_password_route,routes::book_again, routes::edit_appointment_route,
            routes::book, routes::book_error, routes::post_book,
            routes::login, routes::post_login,
            routes::register, routes::register_post,
            routes::logout
        ])
	.mount("/static", rocket::fs::FileServer::from("static"))
}
