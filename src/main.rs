pub mod api;
pub mod cli;
pub mod instance;

use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = cli::Hol::from_args();

    cli::start(opt).await?;
    Ok(())
}
