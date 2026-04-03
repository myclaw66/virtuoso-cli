use crate::config::Config;
use crate::error::Result;
use crate::transport::tunnel::SSHClient;

pub fn run() -> Result<()> {
    let cfg = Config::from_env()?;

    if let Some(state) = crate::models::TunnelState::load()? {
        println!(
            "stopping tunnel on port {} (pid {})...",
            state.port, state.pid
        );

        #[cfg(unix)]
        {
            let result = unsafe { libc::kill(state.pid as i32, libc::SIGTERM) };
            if result == 0 {
                println!("tunnel stopped");
            } else {
                println!("warning: could not kill process {}", state.pid);
            }
        }

        #[cfg(not(unix))]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &state.pid.to_string(), "/F"])
                .output();
            println!("tunnel stop requested");
        }

        if !cfg.keep_remote_files {
            let client = SSHClient::from_env(cfg.keep_remote_files)?;
            let _ = client.run_command("rm -rf /tmp/virtuoso_bridge");
        }

        crate::models::TunnelState::clear()?;
        println!("state cleared");
    } else {
        println!("no running tunnel found");
    }

    Ok(())
}
