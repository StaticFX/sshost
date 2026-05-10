use std::env::home_dir;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Error};

pub struct Connection {
    host: String,
    host_name: String,
    user: String,
    port: u16,
}

fn get_ssh_entries() {
    if let Ok(file) = get_config() {
        let buf = BufReader::new(file);
        for line in buf.lines() {
            let raw_entry = line.unwrap();
        }
    }
}

fn get_config() -> io::Result<File> {
    let suffix = ".ssh/config";
    let ssh_dir = format!("{}{suffix}", home_dir().unwrap().display());
    File::open(ssh_dir)

}

