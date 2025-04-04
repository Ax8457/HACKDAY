use sha2::Sha256;
use hmac::{Hmac, Mac};
use base64::{engine::general_purpose, Engine as _};
use std::{thread, time};
use crate::utilities::verify;
//compute Auth tag
pub fn compute_tokenHMAC(key: &[u8], token: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new(key.into());
    mac.update(token);
    mac.finalize().into_bytes().to_vec()
}

//Chech received token
pub fn receive_token(key: &[u8], plaintext_token: &[u8], authentication_tag: &[u8]) -> bool {
    let mut mac = Hmac::<Sha256>::new(key.into()); //format conversion
    mac.update(plaintext_token);
    let computed_tag = mac.finalize().into_bytes();
    
    verify(&computed_tag, authentication_tag)
    
}

//decode token
pub fn decode_token(token: &str) -> Option<(String, Vec<u8>)> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 2 {
        return None; 
    }
    let plaintext_token_b64 = parts[0];
    let auth_tag_b64 = parts[1];
    //b64 decode
    let plaintext_token_bytes = general_purpose::STANDARD.decode(plaintext_token_b64).ok()?;
    let auth_tag_bytes = general_purpose::STANDARD.decode(auth_tag_b64).ok()?;
    // Convert json payload
    let plaintext_token = String::from_utf8(plaintext_token_bytes).ok()?;
    Some((plaintext_token, auth_tag_bytes))
}

//chack admin ppt 
pub fn is_admin(token_payload: &str) -> bool {
    token_payload.contains("'role':'admin'")
}
