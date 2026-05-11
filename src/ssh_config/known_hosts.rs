use std::env::home_dir;
use std::fs;

/// Parse ~/.ssh/known_hosts and return hostnames/IPs that are not hashed
pub fn get_known_hosts() -> Vec<String> {
    let path = format!("{}/.ssh/known_hosts", home_dir().unwrap().display());
    let content = match fs::read_to_string(path) {
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
