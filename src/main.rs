mod images;

use chrono::{NaiveDateTime, Utc};
use clap::Parser;
use log::{error, info};
use simple_logger::SimpleLogger;
use std::process::exit;

use crate::images::DockerActions;

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

    /// where is located the docker socket (can be a UNIX socket or TCP protocol)
    #[clap(short, long, default_value = "/var/run/docker.sock")]
    socket: String,
}

#[derive(Debug)]
pub struct DateArgs {
    start: i64,
    stop: Option<i64>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let logger = SimpleLogger::new()
        .without_timestamps()
        .with_level(log::LevelFilter::Info);

    if let Some(e) = logger.init().err() {
        eprintln!("failed to initialize logger: {}", e);
        exit(1);
    }

    let actions = DockerActions::new(
        args.socket,
        args.repository,
        args.tags.map_or(vec![], |t| t),
        args.date.map_or(
            DateArgs {
                start: Utc::now().timestamp() - TWO_DAYS_TIMESTAMP,
                stop: None,
            },
            |d| d,
        ),
    );

    let images = match actions.get().await {
        Ok(i) => i,
        Err(e) => {
            error!("failed to retrieve docker images: {}", e);
            exit(1);
        }
    };

    let saved = match actions.delete(actions.filter(images), args.dry_run).await {
        Ok(s) => s,
        Err(e) => {
            error!("failed to retrieve docker images: {}", e);
            exit(1);
        }
    };

    info!(
        "Total disk space saved: {}",
        if saved / 1000_000 >= 1000 {
            format!("{:.2}GB", saved as f64 / 1000_000_000.0)
        } else {
            format!("{:.2}MB", saved as f32 / 1000_000.0)
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
