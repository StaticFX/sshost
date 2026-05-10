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

fn build_entry(host: &SSHConfig) -> String {
    let mut entry = format!("\nHost {}\n", host.host);
    entry.push_str(&format!("    Hostname {}\n", host.hostname));

    if let Some(user) = &host.username {
        if !user.is_empty() {
            entry.push_str(&format!("    User {user}\n"));
        }
    }

    if let Some(port) = host.port {
        if port != 22 {
            entry.push_str(&format!("    Port {port}\n"));
        }
    }

    if let Some(auth) = &host.auth {
        match auth {
            AuthMethod::Password(_) => {
                entry.push_str("    PasswordAuthentication yes\n");
            }
            AuthMethod::Key(path) => {
                entry.push_str(&format!("    IdentityFile {path}\n"));
            }
        }
    }

    entry
}

pub fn write_new_host(host: &SSHConfig) -> io::Result<()> {
    let file = get_ssh_config()?;
    let mut writer = BufWriter::new(file);
    writer.write_all(build_entry(host).as_bytes())?;
    Ok(())
}
