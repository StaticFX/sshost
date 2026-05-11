use std::env::home_dir;
use std::fs;
use std::io;

fn known_hosts_path() -> String {
    format!("{}/.ssh/known_hosts", home_dir().unwrap().display())
}

/// Parse ~/.ssh/known_hosts and return hostnames/IPs that are not hashed
pub fn get_known_hosts() -> Vec<String> {
    let content = match fs::read_to_string(known_hosts_path()) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut hosts = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('|') {
            continue; // skip comments and hashed entries
        }
        if let Some(host_part) = line.split_whitespace().next() {
            // host_part can be "hostname" or "hostname,ip" or "[hostname]:port"
            for h in host_part.split(',') {
                let h = h.trim_start_matches('[');
                let h = if let Some(idx) = h.find(']') {
                    &h[..idx]
                } else {
                    h
                };
                if !h.is_empty() && !hosts.contains(&h.to_string()) {
                    hosts.push(h.to_string());
                }
            }
        }
    }
    hosts
}

/// Remove all lines from known_hosts that contain the given hostname
pub fn remove_known_host(hostname: &str) -> io::Result<()> {
    let path = known_hosts_path();
    let content = fs::read_to_string(&path)?;

    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return true; // keep comments and blank lines
            }
            if let Some(host_part) = trimmed.split_whitespace().next() {
                // Check if this line matches the hostname
                for h in host_part.split(',') {
                    let h = h.trim_start_matches('[');
                    let h = if let Some(idx) = h.find(']') {
                        &h[..idx]
                    } else {
                        h
                    };
                    if h == hostname {
                        return false; // remove this line
                    }
                }
            }
            true
        })
        .collect();

    fs::write(&path, filtered.join("\n") + "\n")
}
