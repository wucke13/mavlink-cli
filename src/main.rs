use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;

use clap::Clap;

use mavlink::common::*;

mod indicatif;
mod parameters;
mod push_pull;
mod skim;

#[derive(Clap)]
#[clap(version, author)]
pub struct Opts {
    /// Mavlink Connection
    /// (tcpout|tcpin|udpout|udpin|udpbcast|serial|file):(ip|dev|path):(port|baud)
    #[clap(short = "c", default_value = "udpbcast:0.0.0.0:14551")]
    mavlink_connection: String,

    #[clap(
        short = "d",
        default_value = "definitions/result/Combined-apm.pdef.json"
    )]
    definitions_file: std::path::PathBuf,

    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Clap)]
#[clap(version, author)]
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
        SubCommand::Pull { ref out_file } => {
            push_pull::pull(&opts, &out_file).unwrap();
        }
        SubCommand::Push { ref out_file } => {
            push_pull::push(&opts, &out_file).unwrap();
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
