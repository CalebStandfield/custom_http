use crate::io;
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
                    write_response(&stream, &ErrorPage::NotFound.status(), &html_page);
                    return;
                }
                ErrorKind::PermissionDenied => {
                    // Should never error when reading the file here, because we pass in a hardcoded error path
                    let html_page =
                        io::file::read_file(&ErrorPage::PermissionDenied.path()).expect("_");
                    write_response(&stream, &ErrorPage::PermissionDenied.status(), &html_page);
                    return;
                }
                _ => {
                    // Something else happened and we should log it
                    eprintln!("Internal server error {}: {}", filename, e);
                    // Should never error when reading the file here, because we pass in a hardcoded error path
                    let html_page =
                        io::file::read_file(&ErrorPage::InternalServerError.path()).expect("_");
                    write_response(
                        &stream,
                        &ErrorPage::InternalServerError.status(),
                        &html_page,
                    );
                    return;
                }
            }
        }
    };

    write_response(&stream, &status, &html_page);
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

fn write_response(mut stream: &TcpStream, status: &String, html_page: &String) {
    let length = html_page.len();

    let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{html_page}");

    stream.write_all(response.as_bytes()).unwrap();
}
