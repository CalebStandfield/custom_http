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

use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:8080";

/// Entry point for the program
fn main() {
    let listener = match TcpListener::bind(ADDRESS) {
        Ok(listener) => listener,
        Err(e) => panic!("Unable to bind to {}: {}", ADDRESS, e),
    };

    // I'm not sure what number of threads is standard
    // However, the book used 4, so for now I will use 4 as well
    let thread_pool = thread_pool::ThreadPool::new(4);

    listener.set_nonblocking(true).expect("Unable to set nonblocking");

    for stream in listener.incoming() {

        match stream {
            Ok(stream) => {
                thread_pool.execute(|| {
                    handle_connection(stream);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Sleeping for a second because I don't want too much CPU usage for
                // this small program.
                std::thread::sleep(std::time::Duration::from_millis(1000));
                continue;
            }
            Err(e) => eprintln!("Another error occurred: {}", e)
        }
    }
}

/// Handles a connection from a client.
///
/// Writes the desired page into the TcpStream.
fn handle_connection(mut stream: TcpStream) {
    let buffer = BufReader::new(&mut stream);
    let request_line = buffer.lines().next().unwrap().unwrap();
    http::response::http_handler(&stream, request_line);
}