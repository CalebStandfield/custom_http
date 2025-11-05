pub mod server;
pub mod thread_pool;
pub mod util;

pub mod http {
    pub mod request;
    pub mod response;
    pub mod headers;
}

pub mod io {
    pub mod nonblocking;
    pub mod file;
}

use std::net::{TcpListener};

const ADDRESS: &str = "127.0.0.1:8080";

/// Entry point for the program
fn main() {
    let listener = match TcpListener::bind(ADDRESS) {
        Ok(listener) => listener,
        Err(e) => panic!("Unable to bind to {}: {}", ADDRESS, e),
    };
}