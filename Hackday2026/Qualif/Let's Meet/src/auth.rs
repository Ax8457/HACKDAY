/*
	Imports
*/
use sha2::Sha256;
use hmac::{Hmac, Mac};
use base64::{engine::general_purpose, Engine as _};
use std::{thread, time};

/*
	HMAC key
*/
pub static key: &[u8] = b"HACKDA7!7ACKDAY!HA7KDA8!HACKDA9!HACKDA1!HACKDAY!HAC3DA7!HACKDAY!";

/*
	Functions used to authenticate users with rocket users guard (Payload authenticated with HMAC)
*/
pub fn compute_tokenHMAC(k: &[u8], token: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new(k.into());
    mac.update(token);
    mac.finalize().into_bytes().to_vec()
}

pub fn check_token(k: &[u8], plaintext_token: &[u8], authentication_tag: &[u8]) -> bool {
    let mut mac = Hmac::<Sha256>::new(k.into()); 
    mac.update(plaintext_token);
    mac.verify(authentication_tag.into()).is_ok()
}

pub fn decode_token(token: &str) -> Option<(String, Vec<u8>)> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 2 {
        return None; 
    }
    let plaintext_token_b64 = parts[0];
    let auth_tag_b64 = parts[1];
    let plaintext_token_bytes = general_purpose::STANDARD.decode(plaintext_token_b64).ok()?;
    let auth_tag_bytes = general_purpose::STANDARD.decode(auth_tag_b64).ok()?;
    let plaintext_token = String::from_utf8(plaintext_token_bytes).ok()?;
    Some((plaintext_token, auth_tag_bytes))
}

pub fn craft_token(username: &str) -> String {
    let token = format!(
        "{{\"username\":\"{}\",\"role\":\"std_user\"}}",
        username
    );
    let plaintext_token = token.as_bytes();
    let auth_tag = compute_tokenHMAC(key, plaintext_token);
    println!("Generated Auth Tag: {:?}", auth_tag);
    let AT_b64encoded = general_purpose::STANDARD.encode(auth_tag);
    let PTT_b64encoded = general_purpose::STANDARD.encode(plaintext_token);
    let token = format!("{}.{}", PTT_b64encoded, AT_b64encoded);
    token
}














