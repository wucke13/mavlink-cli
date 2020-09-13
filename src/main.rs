use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;

use clap::Clap;

use mavlink::common::*;

mod dump;
mod indicatif;
mod parameters;
mod skim;

#[derive(Clap)]
#[clap(version, author)]
pub struct Opts {
    /// Mavlink Connection
    /// (tcpout|tcpin|udpout|udpin|udpbcast|serial|file):(ip|dev|path):(port|baud)
    #[clap(short = "c", default_value = "udpbcast:0.0.0.0:14551")]
    mavlink_connection: String,

    #[clap(short = "d", default_value = env!("DEFAULT_DEFINITION_FILE"))]
    definitions_file: std::path::PathBuf,

    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Clap)]
#[clap(version, author)]
pub enum SubCommand {
    Interactive {},
    Dump {
        #[clap()]
        out_file: std::path::PathBuf,
    },
    Info {
        #[clap()]
        search_term: String,
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

    match opts.cmd {
        SubCommand::Info { search_term } => {
            let progress = indicatif::new_spinner("parsing definitions");
            let params = parameters::parse(&opts.definitions_file).unwrap();
            progress.finish_with_message("done parsing");
            progress.finish();
            let progress = indicatif::new_spinner(&format!("searching for {}", search_term));
            match params.get(&search_term) {
                Some(v) => {
                    progress.finish_with_message("found:");
                    println!("{}", v)
                }
                None => progress.abandon_with_message("did not find anything"),
            }
        }
        SubCommand::Dump { ref out_file } => {
            dump::dump(&opts, &out_file).unwrap();
        }
        SubCommand::Interactive {} => {
            let progress = indicatif::new_spinner("parsing definitions");
            let params = parameters::parse(&opts.definitions_file).unwrap();
            progress.finish_with_message("done parsing");
            progress.finish();

            skim::run(params.values().cloned().collect());
        }
        _ => {}
    }
}
