mod images;

use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use log::{error, info};
use simple_logger::SimpleLogger;
use std::process::{exit, Command, Stdio};

use crate::images::process_imgs;

const DOCKER_BIN: &str = "docker";
const TWO_DAYS_TIMESTAMP: i64 = 172_800;

/// Clear docker images from
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// filter by date.
    ///
    /// Can filter by a minimum age $DATE or from $START|$STOP (format example: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS) [default: $NOW - 2d]
    #[clap(short, long, parse(try_from_str = parse_user_date))]
    date: Option<DateArgs>,

    /// filter by repository name
    repository: Option<String>,

    /// add tags exclusion
    #[clap(short, long)]
    tags: Option<Vec<String>>,

    /// image cleanup will not be triggered [default: false]
    #[clap(long, takes_value = false)]
    dry_run: bool,

    /// force image removal [default: false]
    #[clap(long, takes_value = false)]
    force: bool,

    /// add more logs [default: false]
    #[clap(short, long, takes_value = false)]
    verbose: bool,
}

#[derive(Debug)]
pub struct DateArgs {
    start: i64,
    stop: Option<i64>,
}

fn main() {
    let args = Args::parse();
    let logger = SimpleLogger::new()
        .without_timestamps()
        .with_level(match args.verbose {
            true => log::LevelFilter::Debug,
            false => log::LevelFilter::Info,
        });

    if let Some(e) = logger.init().err() {
        eprintln!("failed to initialize logger: {}", e);
        exit(1);
    }

    let (ids, saved_size) = process_imgs(
        args.repository,
        args.tags.map_or(vec![], |tags| tags),
        args.date.map_or(
            DateArgs {
                start: Utc::now().timestamp() - TWO_DAYS_TIMESTAMP,
                stop: None,
            },
            |d| d,
        ),
    );

    if args.dry_run {
        info!("dry run activated");
    } else {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.arg("rmi");

        if args.force {
            info!("\"--force\" flag set");
            cmd.arg("--force");
        }

        if ids.len() == 0 {
            info!("nothing to do...");
            return;
        }

        if args.verbose {
            info!("trigger \"docker rmi\" command");
        }

        match cmd.args(&ids).stdout(Stdio::null()).status() {
            Ok(s) => {
                if !s.success() {
                    error!("failed to delete images. Please checkout STDERR")
                }

                info!("images deleted!")
            }
            Err(e) => error!("docker command failed: {}", e),
        };
    }

    if args.dry_run {
        info!("deleted images: {:#?}", ids);
    }
    info!(
        "Total disk space saved: {}",
        if saved_size / 1000.0 > 1.0 {
            format!("{:.2}GB", saved_size / 1000.0)
        } else {
            format!("{:.2}MB", saved_size)
        }
    );
}

fn parse_user_date(date: &str) -> Result<DateArgs, &'static str> {
    if date.contains("|") {
        let dates: Vec<&str> = date.split("|").collect();

        return Ok(DateArgs {
            start: format_user_date(dates[0]),
            stop: Some(format_user_date(dates[1])),
        });
    }

    Ok(DateArgs {
        start: format_user_date(date),
        stop: None,
    })
}

fn format_user_date(user_date: &str) -> i64 {
    let date = if user_date.contains("T") {
        user_date.to_string()
    } else {
        format!("{}T00:00:00", user_date)
    };

    NaiveDateTime::parse_from_str(&date, "%FT%T")
        .map_or(Utc::now().timestamp() - TWO_DAYS_TIMESTAMP, |d| {
            d.timestamp()
        })
}
