use std::{thread, time};

pub fn verify(expected: &[u8], received: &[u8]) -> bool {
    if expected.len() != received.len() {
        return false;
    }
    for (a, b) in expected.iter().zip(received.iter()) {
        if a != b {
            return false;
        }
        println!("Byte is valid.");
        thread::sleep(time::Duration::from_millis(10)); //longer check to make bruteforce attacks harder
    }
    true
}

