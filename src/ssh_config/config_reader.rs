use ssh2_config::{ParseRule, SshConfig};
use std::env::home_dir;
use std::fs::File;
use std::io;
use std::io::BufReader;

pub enum AuthMethod {
    Password,
    Key,
}
pub struct SSHConfig {
    host: String,
    hostname: String,
    username: String,
    port: u16,
    auth: AuthMethod,
}

pub fn get_ssh_entries() -> Vec<SSHConfig> {
    let suffix = ".ssh/config";
    let ssh_dir = format!("{}{suffix}", home_dir().unwrap().display());
    let mut reader = BufReader::new(File::open(ssh_dir).unwrap());
    let config = (SshConfig::default())
        .parse(&mut reader, ParseRule::STRICT)
        .unwrap();
    config
        .get_hosts()
        .iter()
        .filter_map(|h| {
            let params = &h.params;

            let host = h
                .pattern
                .iter()
                .filter(|p| !p.negated)
                .map(|p| p.pattern.clone())
                .collect::<Vec<String>>()
                .join(" ");

            if host.is_empty() || host == "*" {
                return None;
            }

            let hostname = params.host_name.clone().unwrap_or_else(|| host.clone());

            let username = params.user.clone().unwrap_or_else(|| "".to_string());

            let port = params.port.unwrap_or(22);

            let auth = if params
                .identity_file
                .as_ref()
                .map(|files| !files.is_empty())
                .unwrap_or(false)
            {
                AuthMethod::Key
            } else {
                AuthMethod::Password
            };

            Some(SSHConfig {
                host,
                hostname,
                username,
                port,
                auth,
            })
        })
        .collect()
}

pub fn write_new_host(host: SSHConfig) -> () {

}
