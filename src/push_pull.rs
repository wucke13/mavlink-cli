use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use chrono::prelude::*;
use mavlink::common::*;

use crate::{to_string, Opts};

use indicatif;

pub fn fetch_parameters(opts: &Opts) -> io::Result<BTreeMap<String, f32>> {
    //let progress = indicatif::new_spinner("establishing connection");
    let conn = mavlink::connect::<MavMessage>(&opts.mavlink_connection).expect("Oh no");
    //progress.set_message("requesting parameters");
    let req_msg = MavMessage::PARAM_REQUEST_LIST(PARAM_REQUEST_LIST_DATA {
        target_component: 0,
        target_system: 0,
    });
    let header = mavlink::MavHeader::default();
    conn.send(&header, &req_msg)?;
    //progress.finish_with_message("done");

    let progress = indicatif::ProgressBar::new(1);
    progress.set_message("requesting messages");
    let mut map = BTreeMap::new();

    while !progress.is_finished() {
        match conn.recv() {
            Ok((_, MavMessage::PARAM_VALUE(p))) => {
                let total = p.param_count.into();
                let param_name = to_string(&p.param_id);
                let value = p.param_value;

                progress.set_length(total);
                progress.set_position(map.len() as u64);
                progress.set_message(&format!("received {}", param_name));

                map.insert(param_name, value);

                if map.len() as u64 == total {
                    progress.finish();
                }
            }
            Err(mavlink::error::MessageReadError::Io(e)) => {
                match e.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        //no messages currently available to receive -- wait a while
                        continue;
                    }
                    _ => {
                        println!("recv error: {:?}", e);
                        break;
                    }
                }
            }
            // messages that didn't get through due to parser errors are ignored
            _ => {}
        }
    }

    Ok(map)
}

/// Dumps the current mavlink configuration
pub fn pull(opts: &Opts, out_file: &Path) -> io::Result<()> {
    let time: DateTime<Local> = Local::now();
    let parameters = fetch_parameters(&opts)?;

    //let progress = indicatif::new_spinner("writing dump");

    let file = File::create(out_file)?;
    writeln!(
        &file,
        "# Generated on {:?} by {}",
        time,
        env!("CARGO_PKG_NAME")
    )?;
    for (param, value) in parameters {
        writeln!(&file, "{},{}", param, value).unwrap();
    }
    //progress.finish_with_message("done writing dump");

    Ok(())
}

/// Dumps the current mavlink configuration
pub fn push(opts: &Opts, in_file: &Path) -> io::Result<()> {
    let time: DateTime<Local> = Local::now();
    let parameters = fetch_parameters(&opts)?;

    //let progress = indicatif::new_spinner("writing dump");

    let file = File::open(in_file)?;
    let file = BufReader::new(file);

    for (line_number, line) in file.lines().enumerate() {
        let line = line?;
        if line.starts_with("#") {
            continue;
        }
        let mut iter = line.split(',');
        let param_name = iter.next().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unable to locate parameter name in line {}", line_number),
            )
        })?;
        let value: f32 = iter
            .next()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unable to locate parameter value in line {}", line_number),
                )
            })?
            .parse()
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unable to parse parameter value in line {}", line_number),
                )
            })?;
    }
    //progress.finish_with_message("done writing dump");

    Ok(())
}
