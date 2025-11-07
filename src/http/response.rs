use std::io::{ErrorKind, Write};
use std::net::TcpStream;
use crate::io;

/// Handles various HTML pages
///
/// # Example
///
/// HTML 200-page
/// HTML 403-page permission denied
/// HTML 404-page not found
/// HTML 500-page internal server error
///
/// As well as other pages which can be added cleanly into this method.
pub fn http_handler(stream: &TcpStream, response: String) {
    let (status, filename) = status_filename(response);

    let html_page = match io::file::read_file(&filename) {
        Ok(html_page) => html_page,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    eprintln!("Error file not found{}: {}", filename, e);
                    let (status, filename) = status_filename(String::from("GET / HTTP/1.1 404 NOT FOUND"));
                    write_response(&stream, status, &filename);
                    return;
                }
                ErrorKind::PermissionDenied => {
                    eprintln!("Error permission denied{}: {}", filename, e);
                    let (status, filename) = status_filename(String::from("GET / HTTP/1.1 403 PERMISSION DENIED"));
                    write_response(&stream, status, &filename);
                    return;
                }
                _ => {
                    eprintln!("Internal server error {}: {}", filename, e);
                    let (status, filename) = status_filename(String::from("GET / HTTP/1.1 500 INTERNAL SERVER ERROR"));
                    write_response(&stream, status, &filename);
                    return;
                }
            }
        }
    };

    write_response(&stream, &status, &html_page);
}


fn status_filename<'a>(response: String) -> (& 'a str, & 'a str) {
    match &response[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "welcome.html"),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    }
}

fn write_response(mut stream: &TcpStream, status: &str, html_page: &str) {
    let length = html_page.len();

    let response =
        format!("{status}\r\nContent-Length: {length}\r\n\r\n{html_page}");

    stream.write_all(response.as_bytes()).unwrap();
}