use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use chrono::prelude::*;
use mavlink::common::*;

use crate::{
    mavlink_stub::{self, MavlinkConnectionHandler},
    parameters::Parameter,
    ui,
    util::*,
};

pub async fn fetch_parameters(
    conn: &mavlink_stub::MavlinkConnectionHandler,
) -> io::Result<Vec<Parameter>> {
    let stream = conn
        .subscribe(mavlink_stub::message_type(&MavMessage::PARAM_VALUE(
            PARAM_VALUE_DATA::default(),
        )))
        .await;

    let req_msg = MavMessage::PARAM_REQUEST_LIST(PARAM_REQUEST_LIST_DATA {
        target_component: 0,
        target_system: 0,
    });

    conn.send_default(&req_msg)?;

    let mut result = Vec::new();

    let bar = ui::bar("fetching parameters");

    let mut param_count = 0;
    for message in smol::stream::block_on(stream) {
        if let MavMessage::PARAM_VALUE(data) = message {
            param_count = param_count.max(data.param_count as u64) as u64;
            bar.set_length(param_count);
            bar.set_position(data.param_index as u64 + 1);
            let name = to_string(&data.param_id);
            let value = data.param_value;

            result.push(Parameter { name, value });

            if bar.position() == param_count {
                bar.finish();
                break;
            }
        }
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

/// Read configuration from vehicle and write to file
pub async fn pull(conn: &MavlinkConnectionHandler, out_file: &Path) -> io::Result<()> {
    let time: DateTime<Local> = Local::now();
    let parameters = fetch_parameters(&conn).await?;

    let progress = ui::spinner("writing dump");

    let file = File::create(out_file)?;
    writeln!(
        &file,
        "# Generated on {:?} by {}",
        time,
        env!("CARGO_PKG_NAME")
    )?;
    for Parameter { name, value } in parameters {
        writeln!(&file, "{},{}", name, value).unwrap();
    }
    progress.finish();

    Ok(())
}

/// Read configuration from file and push to vehicle
pub async fn push(conn: &MavlinkConnectionHandler, in_file: &Path) -> io::Result<()> {
    let progress = ui::spinner("applying parameters");

    let file = File::open(in_file)?;
    let file = BufReader::new(file);

    for (line_number, line) in file.lines().enumerate() {
        let line = line?;
        if line.starts_with('#') {
            continue;
        }
        let mut iter = line.split(',');
        let name = iter
            .next()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unable to locate parameter name in line {}", line_number),
                )
            })?
            .to_string();
        let value = iter
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

        progress.set_message(&format!("applying {}", name));
        let param = Parameter { name, value };
        param.push(&conn).await?;
    }
    progress.finish();

    Ok(())
}
