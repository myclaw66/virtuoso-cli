use crate::client::bridge::VirtuosoClient;
use crate::config::Config;
use crate::error::Result;
use crate::models::TunnelState;
use crate::spectre::runner::SpectreSimulator;

pub fn run() -> Result<()> {
    let cfg = Config::from_env()?;

    println!("=== Virtuoso CLI Status ===\n");

    println!("config:");
    println!(
        "  remote host: {}",
        if cfg.is_remote() {
            &cfg.remote_host
        } else {
            "local"
        }
    );
    if let Some(ref user) = cfg.remote_user {
        println!("  remote user: {user}");
    }
    println!("  port: {}", cfg.port);
    if let Some(ref jump) = cfg.jump_host {
        println!("  jump host: {jump}");
    }
    println!("  timeout: {}s", cfg.timeout);
    println!();

    if let Some(state) = TunnelState::load()? {
        println!("tunnel:");
        println!("  port: {}", state.port);
        println!("  pid: {}", state.pid);
        println!("  remote host: {}", state.remote_host);

        let alive = is_port_open(state.port);
        println!("  port reachable: {}", if alive { "yes" } else { "no" });

        if cfg.is_remote() && state.remote_host != cfg.remote_host {
            println!(
                "  warning: remote host mismatch (expected {})",
                cfg.remote_host
            );
        }
    } else {
        println!("tunnel: not running");
    }
    println!();

    if is_port_open(cfg.port) {
        let vc = VirtuosoClient::local("127.0.0.1", cfg.port, cfg.timeout);
        match vc.test_connection(Some(5)) {
            Ok(true) => println!("daemon: responsive"),
            Ok(false) => println!("daemon: responded unexpectedly"),
            Err(e) => println!("daemon: not reachable ({e})"),
        }
    } else {
        println!("daemon: no tunnel, cannot reach daemon");
    }
    println!();

    if cfg.is_remote() {
        match SpectreSimulator::from_env() {
            Ok(sim) => match sim.check_license() {
                Ok(info) => {
                    println!("spectre license:");
                    for line in info.lines() {
                        println!("  {line}");
                    }
                }
                Err(e) => println!("spectre license: check failed ({e})"),
            },
            Err(e) => println!("spectre: not configured ({e})"),
        }
    }
    println!();

    Ok(())
}

fn is_port_open(port: u16) -> bool {
    use std::net::TcpStream;
    TcpStream::connect(format!("127.0.0.1:{port}")).is_ok()
}
