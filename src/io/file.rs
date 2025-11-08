use std::fs;

pub fn read_file_bytes(filename: &str) -> std::io::Result<Vec<u8>> {
    fs::read(filename)
}
