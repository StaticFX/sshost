# SSHost

```
/ _\/ _\  /\  /\___  ___| |_
\ \ \ \  / /_/ / _ \/ __| __|
_\ \_\ \/ __  / (_) \__ \ |_
\__/\__/\/ /_/ \___/|___/\__|
```

A terminal UI for managing SSH connections, built with [Ratatui](https://github.com/ratatui/ratatui).

SSHost reads your `~/.ssh/config` and gives you a fast, keyboard-driven interface to connect to hosts, manage keys, set up tunnels, and keep your SSH config tidy.

## Features

- **Connect** - browse, filter, and connect to hosts from your SSH config
- **Reachability testing** - check which hosts are online with a TCP probe
- **Host management** - add, edit, and delete SSH host entries
- **SSH tunnels** - configure `LocalForward` and `RemoteForward` per host (2-step config flow)
- **Key generation** - generate ed25519, RSA, or ECDSA keys with overwrite protection
- **Key upload** - deploy public keys to servers via `ssh-copy-id`
- **known_hosts** - browse, import, and remove entries from `~/.ssh/known_hosts`
- **Connection history** - tracks when you last connected to each host

## Install

### From source (recommended)

Requires [Rust](https://www.rust-lang.org/tools/install) (1.85+).

```sh
git clone https://github.com/your-username/sshost.git
cd sshost
cargo install --path .
```

This installs the `sshost` binary to `~/.cargo/bin/`. Make sure it's in your `PATH`.

### Build without installing

```sh
cargo build --release
./target/release/sshost
```

## Usage

```sh
sshost
```

### Keyboard shortcuts

#### Main menu

| Key | Action |
|-----|--------|
| `Up/Down` | Navigate options |
| `Enter` | Select |
| `q` | Quit |

#### Connection list

| Key | Action |
|-----|--------|
| `Up/Down` | Navigate hosts |
| `Enter` | Connect via SSH |
| `/` | Filter hosts |
| `e` | Edit host config |
| `d` | Delete host |
| `t` | Test reachability |
| `Esc` | Back |
| `q` | Quit |

#### Form screens (configure, keygen, upload)

| Key | Action |
|-----|--------|
| `Tab/Down` | Next field |
| `Shift+Tab/Up` | Previous field |
| `Left/Right` | Cycle selector fields (key type, identity file) |
| `Enter` | Submit / next step |
| `Esc` | Back |

## How it works

SSHost is a thin wrapper around your existing SSH tooling:

- **Connections** are read from and written to `~/.ssh/config`
- **SSH sessions** are started by spawning `ssh` with the right flags
- **Key generation** uses `ssh-keygen`
- **Key upload** uses `ssh-copy-id`
- **Tunnels** are stored as `LocalForward`/`RemoteForward` in your SSH config, so they activate automatically on connect
- **History** is stored in `~/.config/sshost/history.json`

No custom state, no daemon, no lock-in. Everything stays in standard SSH config format.

## Dependencies

- Rust 1.85+ (2024 edition)
- `ssh`, `ssh-keygen`, `ssh-copy-id` (standard OpenSSH tools)

## License

MIT
