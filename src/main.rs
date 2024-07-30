mod date;

use chrono::{DateTime, Datelike, FixedOffset, Utc};
use clap::Parser;
use date::get_past_date;
use serde::Deserialize;
use serde_json;
use std::{
    num::ParseFloatError,
    process::{exit, Command, Stdio},
};

const DOCKER_BIN: &str = "docker";

/// Clear docker images from
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// filter by repository name (ISO 8601) [default: $NOW - 2 days]
    #[clap(short, long)]
    date: Option<DateTime<Utc>>,

    /// filter by repository name
    #[clap(short, long)]
    repository: Option<String>,

    /// add tags exclusion
    /// Example: -t 1.1.0 -t release
    #[clap(short, long)]
    tags: Option<Vec<String>>,

    /// image cleanup will not be triggered
    #[clap(long, takes_value = false)]
    dry_run: bool,

    /// should docker force image removal (it may create orphan images)
    #[clap(long, takes_value = false)]
    force: bool,

    /// add more logs
    #[clap(short, long, takes_value = false)]
    verbose: bool,
}

#[derive(Deserialize, Debug)]
struct Image {
    #[serde(with = "date", rename = "CreatedAt")]
    created_at: DateTime<FixedOffset>,
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Tag")]
    tag: String,
    #[serde(rename = "Size")]
    size: String,
}

fn main() {
    let args = Args::parse();

    let stdout = get_images(args.repository);

    let s_data = std::str::from_utf8(&stdout).unwrap();
    let mut images: Vec<&str> = s_data.split("\n").collect();
    // * remove last empty line
    images.remove(images.len() - 1);

    if images.len() == 0 {
        println!("No images found for current timestamp and/or repository");
        return;
    }

    let now = Utc::now();
    let past_date = if let Some(date) = args.date {
        date
    } else {
        get_past_date(now.year(), now.month(), now.day(), 2).and_hms(1, 0, 0)
    };

    let tags = if let Some(t) = args.tags { t } else { vec![] };
    let mut ids = vec![];
    let mut saved_size: f32 = 0.0;

    for img in images {
        let image: Image = serde_json::from_str(img).unwrap();
        if image.created_at.timestamp() <= past_date.timestamp() {
            if !tags.contains(&image.tag) {
                ids.push(image.id);

                saved_size += if image.size.contains("KB") {
                    image
                        .size
                        .replace("KB", "")
                        .parse::<f32>()
                        .unwrap_or_else(failed_convert_size)
                        / 1000 as f32
                } else if image.size.contains("MB") {
                    image
                        .size
                        .replace("MB", "")
                        .parse::<f32>()
                        .unwrap_or_else(failed_convert_size)
                } else if image.size.contains("GB") {
                    image
                        .size
                        .replace("GB", "")
                        .parse::<f32>()
                        .unwrap_or_else(failed_convert_size)
                        * 1000 as f32
                } else {
                    eprintln!("Unknown size identification: {}", image.size);
                    exit(1);
                }
            }
        }
    }

    if args.dry_run {
        println!("dry run activated");
    } else {
        let mut cmd = Command::new(DOCKER_BIN);
        cmd.arg("rmi");

        if args.force {
            println!("\"--force\" flag set");
            cmd.arg("--force");
        }

        if ids.len() == 0 {
            println!("nothing to do...");
            return;
        }

        if args.verbose {
            println!("trigger \"docker rmi\" command");
        }

        match cmd.args(&ids).stdout(Stdio::null()).status() {
            Ok(s) => {
                if !s.success() {
                    eprintln!("failed to delete images. Please checkout STDERR")
                }

                println!("images deleted!")
            }
            Err(e) => eprintln!("docker command failed: {}", e),
        };
    }

    if args.verbose || args.dry_run {
        println!("deleted images: {:#?}", ids);
    }
    println!(
        "Total disk space saved: {}",
        if saved_size / 1000 as f32 > 1 as f32 {
            format!("{:.2}GB", saved_size / 1000.0)
        } else {
            format!("{:.2}MB", saved_size)
        }
    );
}

fn get_images(repo: Option<String>) -> Vec<u8> {
    let mut cmd = Command::new(DOCKER_BIN);
    cmd.arg("images");

    if let Some(repo) = repo {
        cmd.arg(repo);
    }

    cmd.args(["--format", "{{json .}}"]);

    match cmd.output() {
        Ok(o) => {
            if !o.status.success() {
                eprintln!("{}", std::str::from_utf8(&o.stderr).unwrap());
                eprintln!("failed to retrieve docker images. Please checkout STDERR");
                exit(1);
            }

            o.stdout
        }
        Err(e) => {
            eprintln!("docker command failed: {}", e);
            exit(1);
        }
    }
}

fn failed_convert_size(e: ParseFloatError) -> f32 {
    eprintln!("failed to convert \"String\" to \"f32\": {}", e);
    exit(1);
}
