use crate::io;
use mime_guess::from_path;
use std::io::{ErrorKind, Write};
use std::net::TcpStream;

enum ErrorPage {
    NotFound,
    PermissionDenied,
    InternalServerError,
}

impl ErrorPage {
    fn path(&self) -> String {
        match self {
            ErrorPage::NotFound => String::from("public/404.html"),
            ErrorPage::PermissionDenied => String::from("public/403.html"),
            ErrorPage::InternalServerError => String::from("public/500.html"),
        }
    }

    fn status(&self) -> String {
        match self {
            ErrorPage::NotFound => String::from("HTTP/1.1 404 NOT FOUND"),
            ErrorPage::PermissionDenied => String::from("HTTP/1.1 403 PERMISSION DENIED"),
            ErrorPage::InternalServerError => String::from("HTTP/1.1 500 INTERNAL SERVER ERROR"),
        }
    }
}

/// Handles various HTML pages
///
/// # Example
///
/// HTML 200-pages
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
                    // Should never error when reading the file here, because we pass in a hardcoded error path
                    let html_page = io::file::read_file(&ErrorPage::NotFound.path()).expect("_");
                    let response = create_http_response(&ErrorPage::NotFound.status(), &ErrorPage::NotFound.path(), &html_page);
                    write_response(&stream, &response);
                    return;
                }
                ErrorKind::PermissionDenied => {
                    // Should never error when reading the file here, because we pass in a hardcoded error path
                    let html_page =
                        io::file::read_file(&ErrorPage::PermissionDenied.path()).expect("_");
                    let response =
                        create_http_response(&ErrorPage::PermissionDenied.status(), &ErrorPage::PermissionDenied.path(), &html_page);
                    write_response(&stream, &response);
                    return;
                }
                _ => {
                    // Something else happened and we should log it
                    eprintln!("Internal server error {}: {}", filename, e);
                    // Should never error when reading the file here, because we pass in a hardcoded error path
                    let html_page =
                        io::file::read_file(&ErrorPage::InternalServerError.path()).expect("_");
                    let response =
                        create_http_response(&ErrorPage::InternalServerError.status(), &ErrorPage::InternalServerError.path() , &html_page);
                    write_response(&stream, &response);
                    return;
                }
            }
        }
    };

    let response = create_http_response(&status, &filename, &html_page);
    write_response(&stream, &response);
}

fn status_filename(response: String) -> (String, String) {
    let parts: Vec<&str> = response.split_whitespace().collect();
    if parts.len() < 2 {
        return (
            String::from("HTTP/1.1 404"),
            String::from("public/404.html"),
        );
    }

    let path = parts[1];

    if path.contains("..") {
        (
            String::from("HTTP/1.1 404"),
            String::from("public/404.html"),
        )
    } else if path == "/" {
        (
            String::from("HTTP/1.1 200 OK"),
            String::from("public/welcome.html"),
        )
    } else {
        let mut path = String::from(path);
        path.push_str(".html");
        (String::from("HTTP/1.1 200 OK"), format!("public{}", path))
    }
}

fn create_http_response(status: &String, filename: &String, html_page: &String) -> String {
    let mime = from_path(filename).first_or_octet_stream();
    let length = html_page.len();
    format!("{status}\r\nContent-Length: {length}\r\nContent-Type: {mime}\r\n\r\n{html_page}")
}

fn write_response(mut stream: &TcpStream, response: &String) {
    stream.write_all(response.as_bytes()).unwrap();
}
