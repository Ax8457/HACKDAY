/*
	Imports
*/
use rocket::serde::{Serialize, Deserialize};
use rocket::form::FromForm;

/*
	Structs
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct User { pub username: String, pub password: String, #[serde(default)] pub app: Vec<String>, pub role: String, pub company: String, #[serde(default)] pub last_book: Vec<String> }

#[derive(Serialize, Deserialize, Debug)]
pub struct Appointment { pub reference: String, pub event_name: String, pub event_details: String, pub month: String, pub day: u32, pub created_by: String, pub members: Vec<String> }

#[derive(FromForm)]
pub struct MonthForm { pub choosenmonth: u8 }

#[derive(FromForm)]
pub struct RegisterForm<'r> { pub username: &'r str, pub password: &'r str, pub company: &'r str }

#[derive(FromForm)]
pub struct LoginForm<'r> { pub username: &'r str, pub password: &'r str }

#[derive(FromForm)]
pub struct BookingForm { pub name: String, pub month: String, pub details: String, pub day: u32, pub members: String }

#[derive(Serialize, Deserialize)]
pub struct ApiRegisterRequest { pub username: String, pub password: String, pub company: String }

#[derive(Serialize, Deserialize)]
pub struct ApiLoginRequest { pub username: String, pub password: String }

#[derive(Serialize, Deserialize)]
pub struct ApiBookingRequest { pub token: String, pub name: String, pub month: String, pub day: u32, pub details: String, pub members: String, pub referer: String }

#[derive(Serialize, Deserialize)]
pub struct TokenPayload { pub username: String, pub role: String }

#[derive(FromForm)]
pub struct AddMemberForm { pub reference: String, pub new_member: String }

#[derive(FromForm)]
pub struct LeaveForm { pub reference: String }

#[derive(FromForm)]
pub struct ChangePasswordForm { pub new_password: String }

#[derive(FromForm)]
pub struct EditAppointmentForm { pub reference: String, pub details: String }

#[derive(FromForm)]
pub struct RebookForm { pub reference: String }
