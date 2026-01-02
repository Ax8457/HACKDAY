/*
https://doc.rust-lang.org/book/ch21-01-single-threaded.html
*/
mod server;
mod nginx;
mod auth;
mod utilities;
use std::{thread, net::{TcpListener, TcpStream}};
use server::start_server;
use nginx::start_server2;

fn main() {
    // Auth server
    thread::spawn(|| {
        println!(" Auth Server running on http://127.0.0.1:8081/");
        start_server();  
    });

    // API, not exposed, aims to return a json to check token before rendering php pages behind nginx reverse proxy
    thread::spawn(|| {
        println!(" Token Validation Server running on http://127.0.0.1:5000/");
        start_server2(); 
    });
    loop {
        //loop for persistent serv
    }
}
