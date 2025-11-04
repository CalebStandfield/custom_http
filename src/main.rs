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

fn main() {
    println!("Hello, https!");
}