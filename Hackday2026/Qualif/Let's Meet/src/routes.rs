/*
	Imports
*/
use rocket::{get, post, uri, State, form::Form, http::{Cookie, CookieJar, SameSite}, response::Redirect};
use rocket_dyn_templates::{Template, context};
use mongodb::{bson::doc, Client};
use futures::stream::TryStreamExt;
use chrono::{Local, Datelike};
use serde_json;
use rocket::Request;
use crate::structs::{User, Appointment, MonthForm, BookingForm, RegisterForm, LoginForm, AddMemberForm, LeaveForm, ChangePasswordForm, EditAppointmentForm, RebookForm };
use crate::AuthenticatedUser; 
use crate::utilities::get_days;

/*
	Routes
*/
#[get("/", rank = 1)]
pub async fn index(user: AuthenticatedUser, db_client: &State<Client>) -> Template {
    let db = db_client.database("hackday");
    let users_col = db.collection::<User>("users");
    let user_data = users_col.find_one(doc! { "username": &user.username }).await.ok().flatten();
    let booked_refs: Vec<String> = user_data.map(|u| u.app).unwrap_or_default()
        .into_iter()
        .map(|r| format!("{}#", r))
        .collect();
    let local = Local::now();
    let month_str = local.format("%b").to_string().replace(".", "").to_uppercase();
    Template::render("index", context! {
        month: &month_str,
        weekday: local.format("%a").to_string(),
        day: local.day(),
        selectedmonth: &month_str,
        days: get_days(local.year(), local.month()),
        username: user.username,
        booked_refs: booked_refs,
    })
}

#[post("/", data = "<form>")]
pub async fn choosemonth(form: Form<MonthForm>, user: AuthenticatedUser, db_client: &State<Client>) -> Template {
    let db = db_client.database("hackday");
    let users_col = db.collection::<User>("users");
    let user_data = users_col.find_one(doc! { "username": &user.username }).await.ok().flatten();
    let booked_refs: Vec<String> = user_data.map(|u| u.app).unwrap_or_default()
        .into_iter()
        .map(|r| format!("{}#", r))
        .collect();
    let local = Local::now();
    let selectedmonth = form.choosenmonth;
    let months = ["", "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];
    let selectedmonth_str = months.get(selectedmonth as usize).unwrap_or(&"Unknown"); 
    Template::render("index", context! { 
        month: local.format("%b").to_string().replace(".", "").to_uppercase(), 
        weekday: local.format("%a").to_string(), 
        day: local.day(), 
        selectedmonth: selectedmonth_str,
        days: get_days(local.year(), selectedmonth as u32),
        username: user.username, 
        booked_refs: booked_refs,
    })
}

#[get("/account?<error>", rank = 1)]
pub async fn account(user: AuthenticatedUser, db_client: &State<Client>, error: Option<String>) -> Result<Template, Redirect> {
    let db = db_client.database("hackday");
    let users_col = db.collection::<User>("users");
    let app_col = db.collection::<Appointment>("appointments");
    if let Ok(Some(u)) = users_col.find_one(doc! { "username": &user.username }).await {
        let filter = doc! { "members": &user.username };
        let mut cursor = app_col.find(filter).await.map_err(|_| Redirect::to(uri!(login)))?;
        let mut active_apps = Vec::new();
        while let Ok(Some(a)) = cursor.try_next().await { 
            active_apps.push(a); 
        }
        let active_refs: Vec<String> = active_apps.iter().map(|a| a.reference.clone()).collect();
        let mut deleted_refs = Vec::new();
        for url in &u.last_book {
            let day = url.split("d=").nth(1).and_then(|s| s.split('&').next());
            let month = url.split("m=").nth(1).and_then(|s| s.split('&').next());

            if let (Some(d), Some(m)) = (day, month) {
                let theoretical_ref = format!("{}-{}{}", user.username, m, d);
                if !active_refs.contains(&theoretical_ref) {
                    deleted_refs.push(theoretical_ref);
                }
            }
        }
        deleted_refs.sort();
        deleted_refs.dedup();
        return Ok(Template::render("account", context! {
            username: u.username,
            company: u.company,
            role: u.role,
            appointments: active_apps,
            deleted_refs: deleted_refs,
            error: error.unwrap_or_default(), 
        }));
    }
    Err(Redirect::to(uri!(login)))
}	

#[post("/book", data = "<form>")]
pub async fn post_book(user: AuthenticatedUser, form: Form<BookingForm>, jar: &CookieJar<'_>) -> Result<Redirect, Template> {   
    let token = jar.get("session_token").map(|c| c.value()).unwrap_or("");
    let client = reqwest::Client::new();
    let res = client.post("http://127.0.0.1:5000/api/appointments/create")
        .json(&serde_json::json!({
            "token": token, 
            "name": form.name, 
            "month": form.month,
            "day": form.day, 
            "details": form.details, 
            "members": form.members,
            "referer": user.referer 
        }))
        .send()
        .await;
    match res {
        Ok(resp) => {
            let status = resp.status().as_u16();
            if status == 201 {
                Ok(Redirect::to(uri!(account(error = Option::<String>::None))))
            } else {
                let error_msg = match status {
                    404 => "Invited user doesn't exist",
                    409 => "Already booked a meeting on this date",
                    _ => "Error during booking"
                };
                Err(Template::render("book", context! {
                    username: user.username,
                    selected_day: form.day,
                    selected_month: form.month.clone(),
                    error: error_msg
                }))
            }
        },
        Err(_) => Err(Template::render("book", context! {
            username: user.username,
            selected_day: form.day,
            selected_month: form.month.clone(),
            error: "API error"
        }))
    }
}

#[post("/register", data = "<form>")]
pub async fn register_post(form: Form<RegisterForm<'_>>) -> Result<Redirect, Template> {
    let client = reqwest::Client::new();
    let res = client.post("http://127.0.0.1:5000/api/auth/register")
        .json(&serde_json::json!({
            "username": form.username,
            "password": form.password,
            "company": form.company
        }))
        .send().await;
    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(Redirect::to(uri!(login)))
            } else if resp.status().as_u16() == 409 {
                Err(Template::render("register", context! { error: "User already exists" }))
            } else {
                Err(Template::render("register", context! { error: "Registration error" }))
            }
        },
        Err(_) => Err(Template::render("register", context! { error: "API error" }))
    }
}

#[post("/login", data = "<form>")]
pub async fn post_login(form: Form<LoginForm<'_>>, jar: &CookieJar<'_>) -> Result<Redirect, Template> {
    let client = reqwest::Client::new();
    let res = client.post("http://127.0.0.1:5000/api/auth/login")
        .json(&serde_json::json!({
            "username": form.username,
            "password": form.password
        }))
        .send().await;
    if let Ok(resp) = res {
        if resp.status().is_success() {
            let token = resp.text().await.unwrap_or_default();
            jar.add(Cookie::build(("session_token", token))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .build());
            return Ok(Redirect::to(uri!(account(error = Option::<String>::None))));
        }
    }
    Err(Template::render("login", context! { error: "Wrong credentials" }))
}

#[post("/add_member", data = "<form>")]
pub async fn add_member(user: AuthenticatedUser, form: Form<AddMemberForm>, db_client: &State<Client>, jar: &CookieJar<'_>) -> Result<Redirect, Template> {
    let client = reqwest::Client::new();
    let token = jar.get("session_token").map(|c| c.value()).unwrap_or("");
    let res = client.post("http://127.0.0.1:5000/api/appointments/add-member")
        .json(&serde_json::json!({
            "token": token,
            "reference": form.reference,
            "new_member": form.new_member
        }))
        .send()
        .await;
    match res {
        Ok(resp) if resp.status() == 200 => Ok(Redirect::to("/account")),
        Ok(resp) => {
            let error_msg = match resp.status().as_u16() {
                403 => "You are not the owner of this appointment.",
                400 => "This user does not exist.",
                _ => "An error occurred while adding the member."
            };
            let db = db_client.database("hackday");
            let users_col = db.collection::<User>("users");
            let app_col = db.collection::<Appointment>("appointments");
            if let Ok(Some(u)) = users_col.find_one(doc! { "username": &user.username }).await {
                let filter = doc! { "reference": { "$in": &u.app }, "members": &user.username };
                let mut cursor = app_col.find(filter).await.map_err(|_| Template::render("error", context! {}))?;
                let mut apps = Vec::new();
                while let Ok(Some(a)) = cursor.try_next().await { 
                    apps.push(a); 
                }
                return Err(Template::render("account", context! {
                    username: u.username,
                    company: u.company,
                    role: u.role,
                    appointments: apps,
                    error: error_msg 
                }));
            }
            Ok(Redirect::to("/login"))
        },
        Err(_) => {
            Err(Template::render("account", context! { 
                username: user.username,
                role: "std_user", 
                error: "API is unreachable." 
            }))
        }
    }
}

#[post("/leave_appointment", data = "<form>")]
pub async fn leave_route(user: AuthenticatedUser, form: Form<LeaveForm>, db_client: &State<Client>, jar: &CookieJar<'_>) -> Result<Redirect, Template> {
    let client = reqwest::Client::new();
    let token = jar.get("session_token").map(|c| c.value()).unwrap_or("");
    let res = client.post("http://127.0.0.1:5000/api/appointments/leave")
        .json(&serde_json::json!({
            "token": token,
            "reference": form.reference
        }))
        .send()
        .await;
    match res {
        Ok(resp) if resp.status() == 200 => Ok(Redirect::to("/account")),
        Ok(_) => {
            let db = db_client.database("hackday");
            let users_col = db.collection::<User>("users");
            let app_col = db.collection::<Appointment>("appointments");

            if let Ok(Some(u)) = users_col.find_one(doc! { "username": &user.username }).await {
                let filter = doc! { "reference": { "$in": &u.app }, "members": &user.username };
                let mut cursor = app_col.find(filter).await.unwrap();
                let mut apps = Vec::new();
                while let Ok(Some(a)) = cursor.try_next().await { apps.push(a); }

                return Err(Template::render("account", context! {
                    username: u.username,
                    company: u.company,
                    role: u.role,
                    appointments: apps,
                    error: "Could not leave the appointment."
                }));
            }
            Ok(Redirect::to("/login"))
        },
        Err(_) => Ok(Redirect::to("/account")), 
    }
}

#[post("/change_password", data = "<form>")]
pub async fn change_password_route(user: AuthenticatedUser, form: Form<ChangePasswordForm>, db_client: &State<Client>, jar: &CookieJar<'_>) -> Result<Redirect, Template> {
    let client = reqwest::Client::new();
    let token = jar.get("session_token").map(|c| c.value()).unwrap_or("");
    let res = client.post("http://127.0.0.1:5000/api/auth/change_password")
        .json(&serde_json::json!({
            "token": token,
            "new_password": form.new_password
        }))
        .send()
        .await;
    match res {
        Ok(resp) if resp.status() == 200 => {
            Ok(Redirect::to("/logout"))
        },
        _ => {
            let db = db_client.database("hackday");
            let users_col = db.collection::<User>("users");
            let app_col = db.collection::<Appointment>("appointments");
            if let Ok(Some(u)) = users_col.find_one(doc! { "username": &user.username }).await {
                let filter = doc! { "reference": { "$in": &u.app }, "members": &user.username };
                let mut cursor = app_col.find(filter).await.unwrap();
                let mut apps = Vec::new();
                while let Ok(Some(a)) = cursor.try_next().await { apps.push(a); }
                return Err(Template::render("account", context! {
                    username: u.username,
                    company: u.company,
                    role: u.role,
                    appointments: apps,
                    error: "Failed to update password."
                }));
            }
            Ok(Redirect::to("/login"))
        }
    }
}

#[post("/book-again", data = "<form>")]
pub async fn book_again(user: AuthenticatedUser, db_client: &State<Client>, form: Form<RebookForm>, jar: &CookieJar<'_>) -> Result<Template, Redirect> {
    let db = db_client.database("hackday");
    let users_col = db.collection::<User>("users");
    let token = jar.get("session_token").map(|c| c.value()).unwrap_or("");
    if let Some(time_part) = form.reference.split('-').nth(1) {
        let month = &time_part[..3]; 
        let day_str = &time_part[3..];
        if let Ok(Some(u)) = users_col.find_one(doc! { "username": &user.username }).await {
            let target_url = u.last_book.iter().find(|url| {
                url.contains(&format!("d={}", day_str)) && url.contains(&format!("m={}", month))
            });
            if let Some(url) = target_url {
                let url_lower = url.to_lowercase();
                if url_lower.contains("localhost") || url_lower.contains("127.0.0.1") || url_lower.contains("0.0.0.0") {
                    return account(user, db_client, Some("SSRF detected: Access to local services is forbidden.".to_string())).await;
                }
                let client = reqwest::Client::new();
                let _ = client.post(url)
                    .header("Cookie", format!("session_token={}", token))
                    .form(&[
                        ("name", "Re-booking"),
                        ("month", month),
                        ("day", day_str),
                        ("details", "Restored via Re-book"),
                        ("members", &user.username),
                    ])
                    .send()
                    .await;
            }
        }
    }
    Err(Redirect::to(uri!(account(error = Option::<String>::None))))
}

#[post("/edit_appointment", data = "<form>")]
pub async fn edit_appointment_route(user: AuthenticatedUser,form: Form<EditAppointmentForm>, jar: &CookieJar<'_>) -> Redirect {
    let token = jar.get("session_token").map(|c| c.value()).unwrap_or("");
    let client = reqwest::Client::new();
    let _ = client.post("http://127.0.0.1:5000/api/appointments/edit")
        .json(&serde_json::json!({
            "token": token,
            "reference": form.reference,
            "details": form.details
        }))
        .send()
        .await;
    Redirect::to(uri!(account(error = Option::<String>::None)))
}

#[get("/register")] pub fn register() -> Template { Template::render("register", context!{}) }
#[get("/login")] pub fn login() -> Template { Template::render("login", context!{}) }
#[get("/logout")] pub fn logout(jar: &CookieJar<'_>) -> Redirect { jar.remove(Cookie::from("session_token")); Redirect::to(uri!(login)) }
#[get("/book?<d>&<m>", rank = 1)] 
pub async fn book(user: AuthenticatedUser, d: u32, m: String) -> Template { 
    Template::render("book", context! { username: user.username, selected_day: d, selected_month: m }) 
}
#[get("/", rank = 2)] pub fn index_error() -> Redirect { Redirect::to(uri!(login)) }
#[get("/account", rank = 2)] pub fn account_error() -> Redirect { Redirect::to(uri!(login)) }
#[get("/book?<d>&<m>", rank = 2)] pub fn book_error(d: u32, m: String) -> Redirect { 
    let _ = (d, m);
    Redirect::to(uri!(login)) 
}
