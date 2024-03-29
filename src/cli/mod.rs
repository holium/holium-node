// use crate::api::InstanceAPI;
use self::node::NodeRunner;
// use self::rooms::RoomsRunner;
use self::urbit::Instance;
use self::urbit::{UrbitInstance, UrbitUpdateOptions};

mod node;
pub mod printer;
// mod rooms;
pub mod tmux;
mod urbit;

use std::process::exit;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Hol {
    #[structopt(name = "hol", about = "A node for P2P applications.")]
    #[structopt(subcommand)]
    pub command: Subcommand,
    /// the identity of the instance
    #[structopt()]
    server_id: String,
    /// http-port for Urbit instance
    #[structopt(short = "p", long = "urbit-port", default_value = "9030")]
    pub urbit_port: u16,

    // the port for the Holium node
    #[structopt(long = "node-port", default_value = "3030")]
    pub node_port: u16,
}

#[derive(StructOpt)]
pub enum Subcommand {
    /// Installs the Urbit binary
    #[structopt(name = "install")]
    Install,
    /// Boots an identity and exits.
    #[structopt(name = "boot")]
    Boot {
        /// Boots a fake ship
        #[structopt(short = "F", long = "fake")]
        fake: bool,

        /// Urbit id keyfile in string form
        #[structopt(short = "G", long = "key")]
        key: Option<String>,
    },
    /// Starts the instance for the ID registered with the node
    #[structopt(name = "start")]
    Start {},

    /// Stops the instance
    #[structopt(name = "stop")]
    Stop {},
    /// Applies a cleaning script to the instance
    #[structopt(name = "clean")]
    Clean {
        /// the cleaning script applied to the instance (pack, meld, chop, pack-meld, pack-meld-chop)
        #[structopt(short = "m", long = "method", default_value = "pack-meld")]
        method: String,
    },
    /// Returns detailed info about the instance
    #[structopt(name = "info")]
    Info {},
    /// Prints logs or attach to stdout from instance
    #[structopt(name = "logs")]
    Logs {
        /// attach to stdout
        #[structopt(short = "a", long = "attach")]
        attach: bool,
        /// number of recent lines to print
        #[structopt(short = "l", long = "lines", default_value = "100")]
        num_of_lines: i32,
    },
    /// Stops and upgrades the instance to latest version of vere or urbit
    #[structopt(name = "upgrade")]
    Upgrade {
        /// should update vere
        #[structopt(short = "v", long = "vere")]
        update_vere: Option<bool>,
        /// attempt ota
        #[structopt(short = "u", long = "urbit")]
        update_urbit: Option<bool>,
        /// update all
        #[structopt(short = "a", long = "all")]
        update_all: Option<bool>,
    },
    /// Lists all apps installed on the instance
    #[structopt(name = "apps")]
    Apps {},
    /// app subcommands
    #[structopt(name = "app")]
    App {
        /// the name of the app
        #[structopt()]
        app_name: String,
        // info about the app
        // TODO: implement
    },
    /// Prints the current version
    #[structopt(name = "version")]
    Version,
    /// Get the current instance access code
    #[structopt(name = "code")]
    Code,
}

pub fn start(opt: Hol) -> std::io::Result<()> {
    let urbit = UrbitInstance;

    match opt.command {
        Subcommand::Install => {
            urbit.download_and_setup("urbit").unwrap();
            exit(0);
        }
        Subcommand::Boot { fake, key, .. } => {
            urbit.download_and_setup("urbit").unwrap();
            urbit
                .boot(&opt.server_id, fake, key, opt.urbit_port)
                .unwrap();
            NodeRunner
                .start(&opt.server_id, opt.node_port, opt.urbit_port)
                .unwrap();
            exit(0);
        }
        Subcommand::Start {} => {
            urbit.start(&opt.server_id, opt.urbit_port.clone()).unwrap();
            NodeRunner
                .start(&opt.server_id, opt.node_port, opt.urbit_port)
                .unwrap();
            // RoomsRunner.start(&opt.server_id).unwrap();
            exit(0);
        }
        Subcommand::Stop {} => {
            urbit.stop(&opt.server_id, opt.urbit_port.clone())?;
            NodeRunner.stop(&opt.server_id).unwrap();
            // RoomsRunner.stop(&opt.server_id).unwrap();
            exit(0);
        }
        Subcommand::Clean { method } => {
            urbit.clean(&opt.server_id, &method).unwrap();
            exit(0);
        }
        Subcommand::Info {} => {
            urbit.info(&opt.server_id).unwrap();
            exit(0);
        }
        Subcommand::Logs {
            attach,
            num_of_lines,
        } => {
            urbit.logs(&opt.server_id, attach, num_of_lines).unwrap();
            exit(0);
        }
        Subcommand::Upgrade {
            update_urbit,
            update_vere,
            update_all,
        } => {
            urbit
                .upgrade(
                    &opt.server_id,
                    UrbitUpdateOptions {
                        update_urbit: update_urbit,
                        update_vere: update_vere,
                        update_all: update_all,
                    },
                )
                .unwrap();
            exit(0);
        }
        Subcommand::Apps {} => {
            urbit.apps(&opt.server_id).unwrap();
            exit(0);
        }
        Subcommand::App { app_name } => {
            urbit.app(&opt.server_id, &app_name).unwrap();
            exit(0);
        }
        Subcommand::Version => {
            let version = env!("CARGO_PKG_VERSION");
            println!("hol version {}", version);
            urbit.version().unwrap();
            exit(0);
        }
        Subcommand::Code => {
            let access_code = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(urbit_api::lens::get_access_code(opt.server_id))
                .unwrap();

            println!("{}", access_code);
            exit(0);
        }
    }
}
