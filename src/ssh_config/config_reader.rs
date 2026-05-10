use ssh2_config::{Host, ParseRule, SshConfig};
use std::env::home_dir;
use std::fs::File;
use std::io::{BufReader, Error};


fn get_ssh_entries() -> Result<Vec<Host>, Error> {
    let suffix = ".ssh/config";
    let ssh_dir = format!("{}{suffix}", home_dir().unwrap().display());
    let mut  reader = BufReader::new(File::open(ssh_dir)?);
    let config = (SshConfig::default()).parse(&mut reader, ParseRule::STRICT).unwrap();
    Ok(config.get_hosts().clone())
}


