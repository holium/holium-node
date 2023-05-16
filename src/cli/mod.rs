use super::urbit;

use std::{
    process::{exit},
    process::Command,
    process::Stdio
};


use term_table::{Table};
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use structopt::StructOpt;
use std::process::Child;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(StructOpt)]
pub struct Hol {
    #[structopt(name = "hol", about = "A node for P2P applications.")]
    #[structopt(subcommand)]
    pub command: Subcommand,
    /// http-port for Urbit instance
    #[structopt(short = "p", long = "urbit-port", default_value = "9030")]
    pub urbit_port: u16,

    #[structopt(long = "node-port", default_value = "3030")]
    pub node_port: u16,
}

#[derive(StructOpt)]
pub enum Subcommand {
    /// Boots a node with the identity provided.
    #[structopt(name = "boot")]
    Boot {
      /// the server id of the Urbit instance
      #[structopt(short="i", long = "server-id")]
      server_id: String,

      /// Boots a fake ship
      #[structopt(short = "F", long = "fake")]
      fake: bool,

      /// Urbit id keyfile in string form
      #[structopt(short = "G", long = "key")]
      key: Option<String>
    },

    /// List all running nodes
    #[structopt(name = "nodes")]
    Nodes,

    /// Attach to instances for logs, metrics, and running commands
    #[structopt(name = "node")]
    Node {
      /// Prints verbose logs
      #[structopt(short = "v", long = "verbose")]
      verbose: bool,

      #[structopt(subcommand)]
      command: NodeCommand,
    },

    /// Prints the current version
    #[structopt(name = "version")]
    Version,
}


#[derive(StructOpt)]
pub enum NodeCommand {
  /// Starts a node with the given server id
  #[structopt(name = "start")]
  Start {
    /// the server id of the Urbit instance
    #[structopt()]
    server_id: String,
  },

  /// Stops a node with the given server id
  #[structopt(name = "stop")]
  Stop {
    /// the server id of the Urbit instance
    #[structopt(short="i", long = "server-id")]
    server_id: String,
  }
}


pub async fn start(opt: Hol) -> std::io::Result<Arc<Mutex<HashMap<String, Child>>>>  {
    let child_processes: Arc<Mutex<HashMap<String, Child>>> = Arc::new(Mutex::new(HashMap::new()));
    match opt.command {
        Subcommand::Boot { server_id, fake, key, .. } => {
            urbit::download_and_setup_binary("urbit").unwrap();
            let mut command = urbit::start_urbit(&server_id, fake, key, opt.urbit_port).unwrap();
            let child = command.stdout(Stdio::null()).spawn().expect("Failed to start Urbit instance");
            child_processes.lock().unwrap().insert(server_id.clone(), child);
            println!("Started Urbit instance with PID {}", child_processes.lock().unwrap()[&server_id].id());
            exit(0);
        },
        Subcommand::Nodes => {
            let output = Command::new("bash")
                .arg("-c")
                .arg("ps -eo pid,comm,args | grep 'urbit' | grep -v grep | grep -v 'serf'")
                .output()
                .expect("Failed to execute command");

            let output = String::from_utf8(output.stdout).expect("Not UTF-8");
            let mut server_pid_map: HashMap<i32, String> = HashMap::new();

            for line in output.lines() {
                let mut parts = line.trim().splitn(3, ' ');
                let pid = parts.next().unwrap().parse::<i32>().unwrap();
                parts.next().unwrap().to_string();
                let args = parts.next().unwrap().to_string();
                let ship = args.split("./urbit").collect::<Vec<&str>>()[1].split("ships/").collect::<Vec<&str>>()[1].split_whitespace().next().unwrap_or("");
                server_pid_map.insert(pid, ship.to_string());
            }
            
            print_table(server_pid_map);
            exit(0);
       },
        Subcommand::Node { command, .. } => {
            match command {
                NodeCommand::Start { server_id } => {
                    let mut command = urbit::start_urbit(&server_id, true, None, opt.urbit_port).unwrap();
                    let child = command.stdout(Stdio::null()).spawn().expect("Failed to start Urbit instance");
                    child_processes.lock().unwrap().insert(server_id.clone(), child);
                    println!("Started Urbit instance with PID {}", child_processes.lock().unwrap()[&server_id].id());
                    exit(0);
                },
                NodeCommand::Stop { server_id } => {
                    let mut instances = child_processes.lock().unwrap();
                    let child = instances.get_mut(&server_id).unwrap();
                    let _ = std::process::Command::new("kill")
                        .arg("-9")
                        .arg(child.id().to_string())
                        .output();
                    println!("\nUrbit instance with server ID {} stopped \n", server_id);
                    exit(0);
                }
            }
       },
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

fn print_table(data: HashMap<i32, String>) {
    let mut table = Table::new();
    table.max_column_width = 30;
    table.style = term_table::TableStyle::thin();
    table.add_row(Row::new(vec![TableCell::new("Server ID"), TableCell::new("PID")]));

    for (server_id, child_id) in data.iter() {
        let row = Row::new(vec![
          TableCell::new_with_alignment(&child_id.to_string(), 1, Alignment::Right),
            TableCell::new_with_alignment(server_id, 1, Alignment::Left),
        ]);

        table.add_row(row);
    }

    println!("{}", table.render());
}