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
    pub local_forward: Option<String>,
    pub remote_forward: Option<String>,
}

fn get_ssh_config() -> io::Result<File> {
    let suffix = ".ssh/config";
    let ssh_dir = format!("{}/{suffix}", home_dir().unwrap().display());
    File::open(ssh_dir)
}

/// Parse LocalForward and RemoteForward lines from the raw SSH config, keyed by host name
fn parse_forwards() -> (
    std::collections::HashMap<String, String>,
    std::collections::HashMap<String, String>,
) {
    let suffix = ".ssh/config";
    let ssh_path = format!("{}/{suffix}", home_dir().unwrap().display());
    let content = match fs::read_to_string(&ssh_path) {
        Ok(c) => c,
        Err(_) => return (std::collections::HashMap::new(), std::collections::HashMap::new()),
    };

    let mut local_forwards = std::collections::HashMap::new();
    let mut remote_forwards = std::collections::HashMap::new();
    let mut current_host: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Host ") {
            current_host = Some(trimmed.trim_start_matches("Host ").trim().to_string());
        } else if let Some(ref host) = current_host {
            let lower = trimmed.to_lowercase();
            if lower.starts_with("localforward ") {
                let value = trimmed.splitn(2, char::is_whitespace).nth(1).unwrap_or("").trim();
                local_forwards.insert(host.clone(), value.to_string());
            } else if lower.starts_with("remoteforward ") {
                let value = trimmed.splitn(2, char::is_whitespace).nth(1).unwrap_or("").trim();
                remote_forwards.insert(host.clone(), value.to_string());
            }
        }
    }
    (local_forwards, remote_forwards)
}

pub fn get_ssh_entries() -> Vec<SSHConfig> {
    let mut reader = BufReader::new(get_ssh_config().unwrap());
    let config = (SshConfig::default())
        .parse(&mut reader, ParseRule::STRICT)
        .unwrap();

    let (local_forwards, remote_forwards) = parse_forwards();

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

            let local_forward = local_forwards.get(&host).cloned();
            let remote_forward = remote_forwards.get(&host).cloned();

            Some(SSHConfig {
                host,
                hostname,
                username: Some(username),
                port: Some(port),
                auth: Some(auth),
                proxy_jump,
                local_forward,
                remote_forward,
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

    if let Some(forward) = &host.local_forward {
        if !forward.is_empty() {
            entry.push_str(&format!("    LocalForward {forward}\n"));
        }
    }

    if let Some(forward) = &host.remote_forward {
        if !forward.is_empty() {
            entry.push_str(&format!("    RemoteForward {forward}\n"));
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
