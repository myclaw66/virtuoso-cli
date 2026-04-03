use crate::client::bridge::VirtuosoClient;
use crate::error::Result;

pub fn run(lib: &str, cell: &str, view: &str, mode: &str) -> Result<()> {
    let client = VirtuosoClient::from_env()?;
    let result = client.open_cell_view(lib, cell, view, mode)?;

    if result.ok() {
        println!("opened: {}/{}/{}/{}", lib, cell, view, mode);
    } else {
        eprintln!("error: {}", result.errors.join("; "));
        std::process::exit(1);
    }
    Ok(())
}
