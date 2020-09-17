use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;

use clap::Clap;

use mavlink::common::*;

use indicatif;

mod parameters;
mod push_pull;
mod skim;
mod ui;

#[derive(Clap)]
#[clap(version, author)]
pub struct Opts {
    /// Mavlink Connection
    /// (tcpout|tcpin|udpout|udpin|udpbcast|serial|file):(ip|dev|path):(port|baud)
    #[clap(short = "c", default_value = "udpbcast:0.0.0.0:14551")]
    mavlink_connection: String,

    #[clap(
        short = "d",
        default_value = "definitions/ArduPilot/result/Combined-apm.pdef.json"
    )]
    definitions_file: std::path::PathBuf,

    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Interactive {},
    Pull {
        #[clap()]
        out_file: std::path::PathBuf,
    },
    Push {
        #[clap()]
        out_file: std::path::PathBuf,
    },
    #[clap()]
    Info {
        #[clap()]
        search_term: Option<String>,
    },
}

/// Extract String from mavlink PARAM_VALUE_DATA
pub fn to_string(input_slice: &[char]) -> String {
    input_slice
        .iter()
        .filter(|c| **c != char::from(0))
        .collect()
}

fn main() {
    let opts: Opts = Opts::parse();

    ui::wait_and_notice("pasing definitions", || {
        parameters::definitions::init_definitions()
    });

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
        }
        SubCommand::Info { .. } => {
            // for as long as the user wants
            let defs = parameters::definitions::all();
            skim::select_definition(&defs)
        }
        SubCommand::Pull { ref out_file } => {
            push_pull::pull(&opts, &out_file).unwrap();
        }
        SubCommand::Push { ref out_file } => {
            push_pull::push(&opts, &out_file).unwrap();
        }
        SubCommand::Interactive {} => {
            // skim::run(params.values().cloned().collect());
        }
        _ => {}
    }
}
