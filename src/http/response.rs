use crate::io;
use mime_guess::{from_path, mime};
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;

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

struct HttpResponse {
    status: String,
    content_type: String,
    body: Body,
}

enum Body {
    Text(String),
    Binary(Vec<u8>),
}

pub fn http_handler(stream: &TcpStream, response: String) {
    let http_response: HttpResponse =  create_http_response(response);
    write_response(&stream, http_response);
}

fn create_http_response(response: String) -> HttpResponse {
    let (mut status, mut filename) = status_filename(response);
    let mut mime = from_path(&filename).first_or_octet_stream();
    let bytes = match io::file::read_file_bytes(&filename) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            status = String::from(ErrorPage::InternalServerError.status());
            filename = String::from(ErrorPage::InternalServerError.path());
            mime = from_path(&filename).first_or_octet_stream();
            io::file::read_file_bytes(&filename).unwrap()
        }

    };

    let body = if mime.type_() == mime::TEXT {
        // Try for text first, if that fails, fall back to binary
        match String::from_utf8(bytes.clone()) {
            Ok(text) => Body::Text(text),
            Err(_) => Body::Binary(bytes),
        }
    } else {
        Body::Binary(bytes)
    };

    HttpResponse {
        status,
        content_type: mime.to_string(),
        body,
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
// pub fn http_handler(stream: &TcpStream, response: String) {
//     let (status, filename) = status_filename(response);
//
//     let html_page = match io::file::read_file(&filename) {
//         Ok(html_page) => html_page,
//         Err(e) => {
//             match e.kind() {
//                 ErrorKind::NotFound => {
//                     // Should never error when reading the file here, because we pass in a hardcoded error path
//                     let html_page = io::file::read_file(&ErrorPage::NotFound.path()).expect("_");
//                     let response = create_http_response(
//                         &ErrorPage::NotFound.status(),
//                         &ErrorPage::NotFound.path(),
//                         &html_page,
//                     );
//                     write_response(&stream, &response);
//                     return;
//                 }
//                 ErrorKind::PermissionDenied => {
//                     // Should never error when reading the file here, because we pass in a hardcoded error path
//                     let html_page =
//                         io::file::read_file(&ErrorPage::PermissionDenied.path()).expect("_");
//                     let response = create_http_response(
//                         &ErrorPage::PermissionDenied.status(),
//                         &ErrorPage::PermissionDenied.path(),
//                         &html_page,
//                     );
//                     write_response(&stream, &response);
//                     return;
//                 }
//                 _ => {
//                     // Something else happened and we should log it
//                     eprintln!("Internal server error {}: {}", filename, e);
//                     // Should never error when reading the file here, because we pass in a hardcoded error path
//                     let html_page =
//                         io::file::read_file(&ErrorPage::InternalServerError.path()).expect("_");
//                     let response = create_http_response(
//                         &ErrorPage::InternalServerError.status(),
//                         &ErrorPage::InternalServerError.path(),
//                         &html_page,
//                     );
//                     write_response(&stream, &response);
//                     return;
//                 }
//             }
//         }
//     };
//
//     let response = create_http_response(&status, &filename, &html_page);
//     write_response(&stream, &response);
// }

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
            String::from(ErrorPage::PermissionDenied.status()),
            String::from(ErrorPage::PermissionDenied.path()),
        )
    } else if path == "/" {
        (
            // Landing page if no path is specified
            String::from("HTTP/1.1 200 OK"),
            String::from("public/welcome.html"),
        )
    } else {
        let mut path = String::from(path);
        path.push_str(".html");
        if !Path::new(&path).exists() {
            return (String::from(ErrorPage::NotFound.status()), String::from(ErrorPage::NotFound.path()))
        }
        (String::from("HTTP/1.1 200 OK"), format!("public{}", path))
    }
}

// fn create_http_response(status: &String, filename: &String, html_page: &String) -> String {
//     let mime = from_path(filename).first_or_octet_stream();
//     let length = html_page.len();
//     format!("{status}\r\nContent-Length: {length}\r\nContent-Type: {mime}\r\n\r\n{html_page}")
// }

fn write_response(mut stream: &TcpStream, http_response: HttpResponse) {
    let status = http_response.status;
    let mime = http_response.content_type;
    let (length, body_bytes): (usize, &[u8]) = match &http_response.body {
        Body::Text(text) => (text.len(), text.as_bytes()),
        Body::Binary(binary) => (binary.len(), binary)
    };

    let header = format!("{status}\r\nContent-Length: {length}\r\nContent-Type: {mime}\r\n\r\n");
    stream.write_all(header.as_bytes()).unwrap();
    stream.write_all(body_bytes).unwrap();
}
