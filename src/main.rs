#![allow(dead_code)]
use std::sync::Arc;

use clap::Clap;

mod mavlink_stub;
mod parameters;
mod push_pull;
mod skim;
mod ui;

/// A tool to interact with MAVLink compatible vehicles.
///
/// Currently the majority of the features is aimed at configuration management. However, it is
/// planned to extend the scope of this tool to other MAVLink related tasks as well, hence the
/// name.
#[derive(Clap)]
#[clap(author, version, about)]
pub struct Opts {
    /// MAVLink connection string.
    /// (tcpout|tcpin|udpout|udpin|udpbcast|serial|file):(ip|dev|path):(port|baud)
    #[clap(
        short = "c",
        long = "connection",
        default_value = "udpbcast:0.0.0.0:14551"
    )]
    mavlink_connection: String,

    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    /// Interactive configuration management
    ///
    /// Starts a fuzzy finder which allows to search through the MAVLink parameters available on
    /// the connected vehicle. Select one ([Return]) or multiple ([Tabulator]) parameters which you
    /// would like to inspect. You can modify them, including sanity checking if metainformation is
    /// avaibable on the parameter.
    Configure,
    /// Pull configuration from the vehicle to a file
    Pull {
        #[clap()]
        out_file: std::path::PathBuf,
    },
    /// Push configuration from a file to the vehicle
    Push {
        #[clap()]
        in_file: std::path::PathBuf,
    },
    /// Browse all parameters with available metainformation
    ///
    /// Starts a fuzzy finder which allow to search through the MAVLink paramters for which
    /// metainformation is available. Select one ([Return]) or multiple ([Tabulator]) parameters
    /// which you would like to inspect. The avaibable metainformation for each parameter is
    /// printed to stdout.
    #[clap()]
    Info {
        #[clap()]
        search_term: Option<String>,
    },
}

fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();

    ui::wait_and_notice("parsing definitions", || {
        parameters::definitions::init_definitions()
    });

    // without async
    match opts.cmd {
        SubCommand::Info { search_term } if search_term.is_some() => {
            if let Some(search_term) = search_term {
                let progress = ui::spinner("looking up message");
                match parameters::definitions::lookup(&search_term) {
                    Some(def) => {
                        progress.finish();
                        let width = std::cmp::min(textwrap::termwidth(), 80);
                        println!("\n{}", def.description(width));
                    }
                    None => progress.abandon(),
                }
            }
            return Ok(());
        }
        SubCommand::Info { .. } => {
            // for as long as the user wants
            let defs = parameters::definitions::all();
            skim::select_definition(&defs);
            return Ok(());
        }
        _ => {}
    }

    smol::block_on(async {
        let conn = Arc::new(mavlink_stub::MavlinkConnectionHandler::new(
            &opts.mavlink_connection,
        )?);

        // spawn background worker
        smol::spawn({
            let conn = conn.clone();
            async move { conn.main_loop().await }
        })
        .detach();

        match opts.cmd {
            SubCommand::Pull { ref out_file } => {
                push_pull::pull(&conn, &out_file).await.unwrap();
            }
            SubCommand::Push { ref in_file } => {
                push_pull::push(&conn, &in_file).await.unwrap();
            }
            SubCommand::Configure => {
                // skim::run(params.values().cloned().collect());
            }
            _ => {}
        };
        Ok(())
    })
}
