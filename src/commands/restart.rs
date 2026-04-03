use crate::error::Result;

pub fn run() -> Result<()> {
    super::stop::run()?;
    println!("---");
    super::start::run()
}
