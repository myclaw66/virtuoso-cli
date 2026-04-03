use crate::client::bridge::VirtuosoClient;
use crate::config::Config;
use crate::error::Result;
use crate::transport::tunnel::SSHClient;

pub fn run() -> Result<()> {
    let cfg = Config::from_env()?;

    if !cfg.is_remote() {
        println!("local mode: no VB_REMOTE_HOST set");
        println!("start virtuoso bridge manually, then connect with:");
        println!("  virtuoso connect --port {}", cfg.port);
        return Ok(());
    }

    println!("starting tunnel to {}...", cfg.remote_host);

    let mut client = SSHClient::from_env(cfg.keep_remote_files)?;
    client.warm(Some(cfg.timeout))?;

    println!("tunnel established on port {}", client.port);
    println!("testing connection to daemon...");

    let vc = VirtuosoClient::from_env()?;
    match vc.test_connection(Some(cfg.timeout)) {
        Ok(true) => println!("daemon is responsive"),
        Ok(false) => println!("warning: daemon responded unexpectedly"),
        Err(e) => println!("warning: could not test daemon: {e}"),
    }

    println!("ready");
    Ok(())
}
