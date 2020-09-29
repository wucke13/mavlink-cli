#![allow(dead_code)]
use async_mavlink::AsyncMavConn;
use clap::Clap;

mod definitions;
mod parameters;
mod push_pull;
mod skim;
mod ui;
mod util;

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
        #[clap()]
        width: Option<usize>,
    },
}

fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();

    ui::wait_and_notice("parsing definitions", definitions::init);

    let default_width = std::cmp::min(textwrap::termwidth(), 80);

    smol::block_on(async {
        let (conn, event_loop) = AsyncMavConn::new(&opts.mavlink_connection)?;

        smol::spawn(async move { event_loop.await }).detach();

        match opts.cmd {
            SubCommand::Info { search_term, width } if search_term.is_some() => {
                if let Some(search_term) = search_term {
                    let progress = ui::spinner("looking up message");
                    match definitions::lookup(&search_term) {
                        Some(def) => {
                            progress.finish();
                            println!("\n{}", def.description(width.unwrap_or(default_width)));
                        }
                        None => progress.abandon(),
                    }
                }
                return Ok(());
            }
            SubCommand::Info { width, .. } => {
                // for as long as the user wants
                for def in skim::select(Box::pin(smol::stream::iter(definitions::all()))).await? {
                    println!("{}", def.description(width.unwrap_or(default_width)));
                }
                return Ok(());
            }
            SubCommand::Pull { ref out_file } => {
                push_pull::pull(&conn, &out_file).await.unwrap();
            }
            SubCommand::Push { ref in_file } => {
                push_pull::push(&conn, &in_file).await.unwrap();
            }
            SubCommand::Configure => {
                //let mut parameters = push_pull::fetch_parameters(&conn);
                loop {
                    for mut param in
                        skim::select(push_pull::stream_parameters(&conn).await?).await?
                    {
                        param.mutate();
                        param.push(&conn).await?;
                    }
                }
            }
        };
        Ok(())
    })
}
