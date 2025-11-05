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

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:8080";

/// Entry point for the program
fn main() {
    let listener = match TcpListener::bind(ADDRESS) {
        Ok(listener) => listener,
        Err(e) => panic!("Unable to bind to {}: {}", ADDRESS, e),
    };

    for stream in listener.incoming() {
        // Since .incoming() never returns a None I think .unwrap is okay for this.
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buffer = BufReader::new(&mut stream);
    let request_line = buffer.lines().next().unwrap().unwrap();
    let (status, filename) = http_handler(&request_line);

    let html_page = match fs::read_to_string(filename) {
        Ok(html_page) => html_page,
        Err(e) => panic!("PANIC FOR NOW, SHOULD FIND MORE GRACEFUL SOLUTION {}", e),
    };
    let length = html_page.len();

    let response =
        format!("{status}\r\nContent-Length: {length}\r\n\r\n{html_page}");

    stream.write_all(response.as_bytes()).unwrap();

}

/// Handles various html pages
///
/// # Example
///
/// Html 200 page
/// Html 404 page not found
///
/// As well as other pages which can be added cleanly into this method.
fn http_handler(response: &String) -> (&str, &str) {
    let (status, filename) = match &response[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "welcome.html"),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };
    (status, filename)
}