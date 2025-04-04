use std::{
    fs,
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread, 
};
use crate::auth::{compute_tokenHMAC, receive_token, decode_token, is_admin};
use base64::{engine::general_purpose, Engine as _};

fn handle_connection(mut stream: TcpStream) {
    let key: &[u8] = b"HACKDAY!HACKDAY!HACKDAY!HACKDAY!HACKDAY!HACKDAY!HACKDAY!HACKDAY!";
    let mut is_admin_user = false;
    let mut token_valid = false;
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
    
    if let Some(token) = received_token {
        println!("Received Cookie: {}", token);
        if let Some((pt_token, auth_tag)) = decode_token(&token) {
            println!("Decoded JSON: {}", pt_token);
            is_admin_user = is_admin(&pt_token);
            println!("Decoded Auth Tag: {:?}", auth_tag);
            token_valid = receive_token(key, pt_token.as_bytes(), &auth_tag);
            if token_valid {
                println!("Authentication Tag (Hex): {}", auth_tag.iter().map(|byte| format!("{:02x}", byte)).collect::<Vec<String>>().join(""));
                println!("[+] Token Valid !");
            } else {
                println!("[x] Invalid or corrupted token!");
            }
        } else {
            println!("[x] Token format incorrect!");
        }
    } else {
        println!("No session_token found in the request. Creating a new one...");
        let plaintext_token: &str = "{'username':'guest', 'role':'guest'}";
        let plaintext_token = plaintext_token.as_bytes();
        let auth_tag = compute_tokenHMAC(key, plaintext_token);
        println!("Generated Auth Tag: {:?}", auth_tag);
        let AT_b64encoded = general_purpose::STANDARD.encode(auth_tag);
        let PTT_b64encoded = general_purpose::STANDARD.encode(plaintext_token);
        let token = format!("{}.{}", PTT_b64encoded, AT_b64encoded);
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Length: {length}\r\n\
            Set-Cookie: session_token={token}\r\n\
            \r\n\
            {contents}",
            length = 0, contents = ""
        );
        stream.write_all(response.as_bytes()).unwrap();
        return;
    }
    
    let page = if is_admin_user { "html/admin.html" } else { "html/hello.html" };
     let contents = if token_valid {
        let page = if is_admin_user { "html/admin.html" } else { "html/hello.html" };
        fs::read_to_string(page).unwrap_or_else(|_| "Page Not Found".to_string())
    } else {
        "Invalid Token".to_string()
    };
    let length = contents.len();
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Length: {length}\r\n\
        \r\n\
        {contents}"
    );
    stream.write_all(response.as_bytes()).unwrap();
}

pub fn start_server() {
    let listener = TcpListener::bind("0.0.0.0:8081").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

