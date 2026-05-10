use ssh2_config::{ParseRule, SshConfig};
use std::env::home_dir;
use std::fs::File;
use std::io;
use std::io::BufReader;

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Password(String),
    Key(String),
}
#[derive(Debug, Clone)]
pub struct SSHConfig {
    pub host: String,
    pub hostname: String,
    pub username: Option<String>,
    pub port: Option<u16>,
    pub auth: Option<AuthMethod>,
}

pub fn get_ssh_entries() -> Vec<SSHConfig> {
    let suffix = ".ssh/config";
    let ssh_dir = format!("{}/{suffix}", home_dir().unwrap().display());

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
                username: Some(username),
                port: Some(port),
                auth: Some(auth("a".to_string())),
            })
        })
        .collect()
}

pub fn write_new_host(host: SSHConfig) -> () {
    let mut host_str = String::from("Host ");
    host_str.push_str(host.host.as_str());

    let mut hostname_str = String::from("Hostname ");
    hostname_str.push_str(host.hostname.as_str());

    if let Some(user) = host.username {
        let mut username_str = String::from("User ");
        username_str.push_str(user.as_str());
    }

    if let Some(port) = host.port {
        let mut port_str = String::from("Port ");
        port_str.push_str(port.to_string().as_str());
    }

    if let Some(auth) = host.auth {
        let mut auth_str = String::from("IdentityFile ");
        //auth_str.push_str(auth.to_string().as_str());
    }
    let mut entry = String::new();
    //entry.push_str("Host ".to_string().join);
}
