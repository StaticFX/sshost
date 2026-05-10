use ssh2_config::{ParseRule, SshConfig};
use std::env::home_dir;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Write};

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

fn get_ssh_config() -> io::Result<File> {
    let suffix = ".ssh/config";
    let ssh_dir = format!("{}/{suffix}", home_dir().unwrap().display());
    File::open(ssh_dir)
}

pub fn get_ssh_entries() -> Vec<SSHConfig> {
    let mut reader = BufReader::new(get_ssh_config().unwrap());
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


fn build_entry(host: SSHConfig) -> String {
    let mut entry = String::from("");

    let mut host_str = String::from("Host ");
    host_str.push_str(host.host.as_str());
    entry.push_str(host_str.as_str());

    let mut hostname_str = String::from("Hostname ");
    hostname_str.push_str(host.hostname.as_str());
    entry.push_str(hostname_str.as_str());

    let mut username_str = String::from("User ");
    if let Some(user) = host.username {
        username_str.push_str(user.as_str());
        entry.push_str(username_str.as_str());
    }

    let mut port_str = String::from("Port ");
    if let Some(port) = host.port {
        port_str.push_str(port.to_string().as_str());
        entry.push_str(port_str.as_str());
    }

    let mut auth_str = String::from("");
    if let Some(auth) = host.auth {
        match auth {
            AuthMethod::Password(_p) =>  {
                auth_str.push_str("PasswordAuthentication yes");
            },
            AuthMethod::Key(p) => {
                auth_str.push_str("IdentityFile ");
                auth_str.push_str(&p)

            },
        }
        entry.push_str(auth_str.as_str());
    }
    entry
}
pub fn write_new_host(host: SSHConfig) -> io::Result<()> {
    let mut writer = BufWriter::new(get_ssh_config()?);
    writer.write_all(build_entry(host).as_bytes())?;
    Ok(())
}
