use crate::client::bridge::VirtuosoClient;
use crate::error::Result;

pub fn run(code: &str, timeout: u64) -> Result<()> {
    let client = VirtuosoClient::from_env()?;
    let result = client.execute_skill(code, Some(timeout))?;

    if result.ok() {
        println!("{}", result.output);
    } else {
        eprintln!("error: {}", result.errors.join("; "));
        std::process::exit(1);
    }
    Ok(())
}
