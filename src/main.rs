mod cli;

use structopt::StructOpt;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = cli::Hol::from_args();

    cli::start(opt).unwrap();
    Ok(())
}
