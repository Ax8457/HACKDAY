/*
   Imports
*/
use rocket::{post, routes, State, http::Status, Data};
use mongodb::{bson::doc, Client};
use crate::auth;
use crate::structs::{ApiLoginRequest, ApiRegisterRequest, ApiBookingRequest, TokenPayload, User, Appointment};
use serde_json;
use tokio::io::AsyncReadExt;

/*
	API routes
*/
#[post("/auth/register", data = "<data>")]
async fn register(data: Data<'_>, db: &State<Client>) -> Status {
    let mut buffer = String::new();
    if data.open(rocket::data::ToByteUnit::megabytes(1)).read_to_string(&mut buffer).await.is_ok() {
        if let Ok(req) = serde_json::from_str::<ApiRegisterRequest>(&buffer) {
            let col = db.database("hackday").collection::<mongodb::bson::Document>("users");
            if let Ok(Some(_)) = col.find_one(doc! { "username": &req.username }).await {
                return Status::Conflict; 
            }
            let new_user = doc! {
                "username": &req.username, "password": &req.password,
                "app": [], "role": "std_user", "company": &req.company
            };
            if col.insert_one(new_user).await.is_ok() { return Status::Created; }
        }
    }
    Status::InternalServerError
}

#[post("/auth/login", data = "<data>")]
async fn login(data: Data<'_>, db: &State<Client>) -> Result<String, Status> {
    let mut buffer = String::new();
    if data.open(rocket::data::ToByteUnit::megabytes(1)).read_to_string(&mut buffer).await.is_ok() {
        if let Ok(req) = serde_json::from_str::<ApiLoginRequest>(&buffer) {
            let col = db.database("hackday").collection::<mongodb::bson::Document>("users");
            if let Ok(Some(user)) = col.find_one(doc! { "username": &req.username }).await {
                if user.get_str("password").unwrap_or("") == req.password {
                    return Ok(auth::craft_token(&req.username));
                }
            }
            return Err(Status::Unauthorized);
        }
    }
    Err(Status::BadRequest)
}

#[post("/appointments/create", data = "<data>")]
async fn book(data: Data<'_>, db: &State<Client>) -> Status {
    let mut buffer = String::new();
    if data.open(rocket::data::ToByteUnit::megabytes(1)).read_to_string(&mut buffer).await.is_ok() {
        if let Ok(req) = serde_json::from_str::<ApiBookingRequest>(&buffer) {
            let referer_url = req.referer.clone(); 
            if let Some((json_str, tag)) = auth::decode_token(&req.token) {
                if auth::check_token(auth::key, json_str.as_bytes(), &tag) {
                    if let Ok(payload) = serde_json::from_str::<TokenPayload>(&json_str) {
                        let db_conn = db.database("hackday");
                        let app_col = db_conn.collection::<mongodb::bson::Document>("appointments");
                        let users_col = db_conn.collection::<mongodb::bson::Document>("users");
                        let mut members: Vec<String> = req.members.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if !members.contains(&payload.username) { members.push(payload.username.clone()); }
                        let filter_existence = doc! { "username": { "$in": &members } };
                        match users_col.count_documents(filter_existence).await {
                            Ok(count) if count == members.len() as u64 => {},
                            _ => return Status::NotFound,
                        }
                        let clean_month = req.month.replace(".", "").to_uppercase();
                        let reference = format!("{}-{}{}", payload.username, clean_month, req.day);
                        if let Ok(Some(_)) = app_col.find_one(doc! { "reference": &reference }).await {
                            return Status::Conflict;
                        }
                        let new_app = doc! {
                            "reference": &reference, 
                            "event_name": &req.name, 
                            "event_details": &req.details,
                            "month": &req.month, 
                            "day": req.day, 
                            "created_by": &payload.username, 
                            "members": &members,
                        };
                        if app_col.insert_one(new_app).await.is_ok() {
                            let _ = users_col.update_one(
                                doc! { "username": &payload.username },
                                doc! { "$addToSet": { "last_book": &referer_url, "app": &reference } }
                            ).await;
                            let others: Vec<String> = members.into_iter()
                                .filter(|m| m != &payload.username)
                                .collect();
                            if !others.is_empty() {
                                let _ = users_col.update_many(
                                    doc! { "username": { "$in": &others } }, 
                                    doc! { "$addToSet": { "app": &reference } }
                                ).await;
                            }
                            return Status::Created;
                        }
                    }
                }
            }
            return Status::Unauthorized;
        }
    }
    Status::BadRequest
}

#[post("/appointments/add-member", data = "<data>")]
async fn add_member_api(data: Data<'_>, db: &State<Client>) -> Status {
    let mut buffer = String::new();
    if data.open(rocket::data::ToByteUnit::megabytes(1)).read_to_string(&mut buffer).await.is_ok() {
        if let Ok(req) = serde_json::from_str::<serde_json::Value>(&buffer) {
            let token = req["token"].as_str().unwrap_or("");
            let r_ref = req["reference"].as_str().unwrap_or("");
            let new_member = req["new_member"].as_str().unwrap_or("");
            if let Some((json_str, tag)) = auth::decode_token(token) {
                if auth::check_token(auth::key, json_str.as_bytes(), &tag) {
                    if let Ok(payload) = serde_json::from_str::<TokenPayload>(&json_str) {
                        let db_conn = db.database("hackday");
                        let app_col = db_conn.collection::<mongodb::bson::Document>("appointments");
                        let users_col = db_conn.collection::<mongodb::bson::Document>("users");
                        if let Ok(Some(app_doc)) = app_col.find_one(doc! { "reference": r_ref }).await {
                            let owner = app_doc.get_str("created_by").unwrap_or("");
                            if owner != payload.username {
                                return Status::Forbidden; 
                            }
                            if let Ok(Some(_)) = users_col.find_one(doc! { "username": new_member }).await {
                                let update_app = app_col.update_one(
                                    doc! { "reference": r_ref },
                                    doc! { "$addToSet": { "members": new_member } }
                                ).await;
                                let update_user = users_col.update_one(
                                    doc! { "username": new_member },
                                    doc! { "$addToSet": { "app": r_ref } }
                                ).await;
                                if update_app.is_ok() && update_user.is_ok() {
                                    return Status::Ok;
                                }
                            } else {
                                return Status::BadRequest; 
                            }
                        } else {
                            return Status::NotFound; 
                        }
                    }
                }
            }
            return Status::Unauthorized;
        }
    }
    Status::InternalServerError
}

#[post("/appointments/leave", data = "<data>")]
async fn leave_appointment(data: Data<'_>, db: &State<Client>) -> Status {
    let mut buffer = String::new();
    if data.open(rocket::data::ToByteUnit::megabytes(1)).read_to_string(&mut buffer).await.is_ok() {
        if let Ok(req) = serde_json::from_str::<serde_json::Value>(&buffer) {
            let token = req["token"].as_str().unwrap_or("");
            let r_ref = req["reference"].as_str().unwrap_or("");
            if let Some((json_str, tag)) = auth::decode_token(token) {
                if auth::check_token(auth::key, json_str.as_bytes(), &tag) {
                    if let Ok(payload) = serde_json::from_str::<TokenPayload>(&json_str) {
                        let db_conn = db.database("hackday");
                        let app_col = db_conn.collection::<mongodb::bson::Document>("appointments");
                        let users_col = db_conn.collection::<mongodb::bson::Document>("users");
                        if let Ok(Some(app_doc)) = app_col.find_one(doc! { "reference": r_ref }).await {
                            let owner = app_doc.get_str("created_by").unwrap_or("");
                            if owner == payload.username {
                                let members = app_doc.get_array("members").ok().cloned();
                                if let Some(members_list) = members {
                                    let _ = users_col.update_many(
                                        doc! { "username": { "$in": members_list } },
                                        doc! { "$pull": { "app": r_ref } }
                                    ).await;
                                }
                                if app_col.delete_one(doc! { "reference": r_ref }).await.is_ok() {
                                    return Status::Ok;
                                }
                            } 
                            else {
                                let _ = app_col.update_one(
                                    doc! { "reference": r_ref },
                                    doc! { "$pull": { "members": &payload.username } }
                                ).await;
                                if users_col.update_one(
                                    doc! { "username": &payload.username },
                                    doc! { "$pull": { "app": r_ref } }
                                ).await.is_ok() {
                                    return Status::Ok;
                                }
                            }
                        } else {
                            return Status::NotFound;
                        }
                    }
                }
            }
            return Status::Unauthorized;
        }
    }
    Status::BadRequest
}

#[post("/auth/change_password", data = "<data>")]
async fn change_password_api(data: Data<'_>, db: &State<Client>) -> Status {
    let mut buffer = String::new(); 
    if data.open(rocket::data::ToByteUnit::megabytes(1)).read_to_string(&mut buffer).await.is_ok() {
        if let Ok(req) = serde_json::from_str::<serde_json::Value>(&buffer) {
            let token = req["token"].as_str().unwrap_or("");
            let new_password = req["new_password"].as_str().unwrap_or("");
            if new_password.is_empty() { 
                return Status::BadRequest; 
            }
            if let Some((json_str, tag)) = auth::decode_token(token) {
                if auth::check_token(auth::key, json_str.as_bytes(), &tag) {
                    if let Ok(payload) = serde_json::from_str::<TokenPayload>(&json_str) {             
                        let db_conn = db.database("hackday");
                        let users_col = db_conn.collection::<mongodb::bson::Document>("users");
                        let filter = doc! { "username": &payload.username };
                        let update = doc! { "$set": { "password": new_password } };
                        match users_col.update_one(filter, update).await {
                            Ok(result) => {
                                if result.matched_count == 0 {
                                    return Status::NotFound;
                                }
                                return Status::Ok;
                            },
                            Err(e) => {
                                return Status::InternalServerError;
                            }
                        }
                    } else {
                        println!("Error parsing json");
                    }
                } else {
                    println!("Invalid signature");
                }
            } else {
                println!("b64 decode failure");
            }
            return Status::Unauthorized;
        }
    }
    Status::BadRequest
}

#[post("/appointments/edit", data = "<data>")]
async fn edit_appointment(data: Data<'_>, db: &State<Client>) -> Status {
    let mut buffer = String::new();
    if data.open(rocket::data::ToByteUnit::megabytes(1)).read_to_string(&mut buffer).await.is_ok() {
        if let Ok(req) = serde_json::from_str::<serde_json::Value>(&buffer) {
            let token = req["token"].as_str().unwrap_or("");
            let r_ref = req["reference"].as_str().unwrap_or("");
            let new_details = req["details"].as_str().unwrap_or("");
            if let Some((json_str, tag)) = auth::decode_token(token) {
                if auth::check_token(auth::key, json_str.as_bytes(), &tag) {
                    if let Ok(payload) = serde_json::from_str::<TokenPayload>(&json_str) {
                        let db_conn = db.database("hackday");
                        let app_col = db_conn.collection::<mongodb::bson::Document>("appointments");
                        if let Ok(Some(app_doc)) = app_col.find_one(doc! { "reference": r_ref }).await {
                            let owner = app_doc.get_str("created_by").unwrap_or("");
                            
                            if owner != payload.username {
                                return Status::Forbidden; 
                            }
                            let update_res = app_col.update_one(
                                doc! { "reference": r_ref },
                                doc! { "$set": { "event_details": new_details } }
                            ).await;

                            if update_res.is_ok() {
                                return Status::Ok;
                            }
                        } else {
                            return Status::NotFound;
                        }
                    }
                }
            }
            return Status::Unauthorized;
        }
    }
    Status::BadRequest
}

#[post("/admin/add-to-app?<new_user>&<reference>")]
pub async fn admin_force_add_post(db_client: &State<mongodb::Client>, new_user: String, reference: String) -> String {
    admin_force_add(db_client, new_user, reference).await
}

#[get("/admin/add-to-app?<new_user>&<reference>")]
pub async fn admin_force_add(db_client: &State<mongodb::Client>,new_user: String,reference: String) -> String {
    let db = db_client.database("hackday");
    let app_col = db.collection::<Appointment>("appointments");
    let user_col = db.collection::<User>("users");
    if let Ok(None) = user_col.find_one(doc!{"username":&new_user}).await {
        return format!("Error: User '{}' doesn't exist",new_user);
    }
    let filter_app = doc!{"reference":&reference};
    match app_col.find_one(filter_app.clone()).await {
        Ok(Some(_)) => {
            let _ = app_col.update_one(filter_app,doc!{"$addToSet":{"members":&new_user}}).await;
            let _ = user_col.update_one(doc!{"username":&new_user},doc!{"$addToSet":{"app":&reference}}).await;
            format!("OK: User '{}' added to appointment '{}'",new_user,reference)
        },
        Ok(None) => format!("Error: Appointment '{}' doesn't exist",reference),
        Err(e) => format!("Error: Database failure ({})",e)
    }
}

/*
	Launch localhost API and attache routes, mongodb, => port 5000
*/
pub async fn launch_api(mongo: Client) -> Result<(),rocket::Error> {
    let config = rocket::Config::figment().merge(("port",5000)).merge(("address","127.0.0.1"));
    rocket::custom(config)
        .manage(mongo)
        .mount("/api",routes![register,login,book,add_member_api,leave_appointment,change_password_api,edit_appointment,admin_force_add,admin_force_add_post])
        .launch()
        .await?;
    Ok(())
}


