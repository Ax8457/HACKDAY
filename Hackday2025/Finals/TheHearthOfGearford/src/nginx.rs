use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use crate::auth::{compute_tokenHMAC, receive_token, decode_token, is_admin};
use base64::{engine::general_purpose, Engine as _};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ValidationResponse {
    is_valid: bool,
    is_admin: bool,
}

pub fn handle_request(mut stream: TcpStream) {
    let key: &[u8] = b"HACKDAY!HACKDAY!HACKDAY!HACKDAY!HACKDAY!HACKDAY!HACKDAY!HACKDAY!";
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let mut received_token = None;
    for line in &http_request {
        if line.starts_with("Cookie:") {
            for cookie in line["Cookie:".len()..].trim().split("; ") {
                if cookie.starts_with("session_token=") {
                    received_token = Some(cookie["session_token=".len()..].to_string());
                    break;
                }
            }
        }
    }

    let mut is_admin_user = false;
    let mut token_valid = false;

    // Validation du token
    if let Some(token) = received_token {
        if let Some((pt_token, auth_tag)) = decode_token(&token) {
            is_admin_user = is_admin(&pt_token);
            token_valid = receive_token(key, pt_token.as_bytes(), &auth_tag);
        }
    }

    // Création de la réponse JSON
    let response = ValidationResponse {
        is_valid: token_valid,
        is_admin: is_admin_user,
    };

    let response_json = serde_json::to_string(&response).unwrap();

    // Réponse HTTP avec JSON
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: application/json\r\n\
        Content-Length: {length}\r\n\
        \r\n\
        {body}",
        length = response_json.len(),
        body = response_json
    );

    stream.write_all(response.as_bytes()).unwrap();
}

pub fn start_server2() {
    let listener = TcpListener::bind("127.0.0.1:5000").unwrap(); 
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_request(stream); 
    }
}

