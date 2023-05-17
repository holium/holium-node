use super::urbit;
mod instance;
pub mod printer;

use std::{
    process::{exit},
};

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
      update_vere: bool,
      /// attempt ota
      #[structopt(short = "u", long = "urbit")]
      update_urbit: bool,
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
}


pub async fn start(opt: Hol) -> std::io::Result<()>  {
    match opt.command {
        Subcommand::Install => {
          urbit::download_and_setup_binary("urbit").unwrap();
          exit(0);
        }
        Subcommand::Boot { fake, key, .. } => {
            urbit::download_and_setup_binary("urbit").unwrap();
            urbit::boot_urbit(&opt.server_id, fake, key, opt.urbit_port).unwrap();
            exit(0);
        }
        Subcommand::Start { } => {
          instance::start_instance(&opt.server_id, opt.urbit_port.clone()).unwrap();
          exit(0);
        }
        Subcommand::Stop {  } => {
          instance::stop_instance(&opt.server_id, opt.urbit_port.clone()).unwrap();
          exit(0);
        }
        Subcommand::Clean {  method } => {
          // TODO: implement
          println!("Cleaning instance {} with method {}", &opt.server_id, method);
          // instance::stop_instance(&opt.server_id, opt.urbit_port.clone()).unwrap();
          exit(0);
        }
        Subcommand::Info { } => {
          // TODO: implement
          println!("Getting info for {}", &opt.server_id);
          // instance::get_info(&opt.server_id).unwrap();
          exit(0);
        }
        Subcommand::Logs { attach, num_of_lines } => {
          // TODO: implement
          println!("Getting logs for {}, attached={}, lines={}", &opt.server_id, attach, num_of_lines);
          // instance::get_logs(&opt.server_id, attach, num_of_lines).unwrap();
          exit(0);
        }
        Subcommand::Upgrade { update_urbit, update_vere } => {
          // TODO: implement
          println!("Upgrading instance {}, update_vere={}, update_urbit={}", &opt.server_id, update_vere, update_urbit);
          exit(0);
        }
        Subcommand::Apps { } => {
          // TODO: implement
          println!("Getting apps for {}", &opt.server_id);
          exit(0);
        }
        Subcommand::App {app_name} => {
          // TODO: implement
          println!("Getting app info {}", &app_name);
          exit(0);
        }
        Subcommand::Version => {
            let version = env!("CARGO_PKG_VERSION");
            println!("hol version {}", version);
            exit(0);
        },
    }
}

// pub fn handle_ctrl_c(child_processes: Arc<Mutex<HashMap<String, Child>>>) {
pub fn handle_ctrl_c() {
    ctrlc::set_handler(move || {
        // for all child processes, kill them
        // let mut instances = child_processes.lock().unwrap();
        // for (server_id, child) in instances.iter_mut() {
        //     let _ = std::process::Command::new("kill")
        //         .arg("-9")
        //         .arg(child.id().to_string())
        //         .output();
        //     println!("\nUrbit instance with server ID {} stopped \n", server_id);

        // }

        // Exiting the program
        std::process::exit(0);

    }).expect("Error setting Ctrl-C handler");
}

// fn print_table(data: HashMap<i32, String>) {
//     let mut table = Table::new();
//     table.max_column_width = 30;
//     table.style = term_table::TableStyle::thin();
//     table.add_row(Row::new(vec![TableCell::new("Server ID"), TableCell::new("PID")]));

//     for (server_id, child_id) in data.iter() {
//         let row = Row::new(vec![
//           TableCell::new_with_alignment(&child_id.to_string(), 1, Alignment::Right),
//             TableCell::new_with_alignment(server_id, 1, Alignment::Left),
//         ]);

//         table.add_row(row);
//     }

//     println!("{}", table.render());
// }