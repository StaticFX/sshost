use ssh2_config::{ParseRule, SshConfig};
use std::env::home_dir;
use std::fs::{self, File};
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
    pub proxy_jump: Option<String>,
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

            let auth = if let Some(files) = params.identity_file.as_ref() {
                if let Some(path) = files.first() {
                    AuthMethod::Key(path.to_string_lossy().to_string())
                } else {
                    AuthMethod::Password(String::new())
                }
            } else {
                AuthMethod::Password(String::new())
            };

            let proxy_jump = params.proxy_jump.as_ref().map(|v| v.join(","));

            Some(SSHConfig {
                host,
                hostname,
                username: Some(username),
                port: Some(port),
                auth: Some(auth),
                proxy_jump,
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

    if let Some(proxy) = &host.proxy_jump {
        if !proxy.is_empty() {
            entry.push_str(&format!("    ProxyJump {proxy}\n"));
        }
    }

    entry
}

pub fn write_new_host(host: &SSHConfig) -> io::Result<()> {
    let suffix = ".ssh/config";
    let ssh_path = format!("{}/{suffix}", home_dir().unwrap().display());
    let file = std::fs::OpenOptions::new().append(true).open(ssh_path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(build_entry(host).as_bytes())?;
    Ok(())
}

pub fn delete_host(host_name: &str) -> io::Result<()> {
    let suffix = ".ssh/config";
    let ssh_path = format!("{}/{suffix}", home_dir().unwrap().display());
    let content = fs::read_to_string(&ssh_path)?;

    let mut result = String::new();
    let mut inside_target_block = false;

    for line in content.lines() {
        if line.starts_with("Host ") {
            let name = line.trim_start_matches("Host ").trim();
            if name == host_name {
                inside_target_block = true;
                continue;
            } else {
                inside_target_block = false;
            }
        }

        if inside_target_block {
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    fs::write(&ssh_path, result)?;
    Ok(())
}

pub fn update_host(old_host_name: &str, new_config: &SSHConfig) -> io::Result<()> {
    delete_host(old_host_name)?;

    let suffix = ".ssh/config";
    let ssh_path = format!("{}/{suffix}", home_dir().unwrap().display());
    let file = std::fs::OpenOptions::new().append(true).open(ssh_path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(build_entry(new_config).as_bytes())?;
    Ok(())
}
