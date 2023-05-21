pub mod cli;
pub mod db;
pub mod instance;
pub mod node;

use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = cli::Hol::from_args();

    cli::start(opt).await?;
    Ok(())
}
