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
const DOCKER_IMGS_CMD: [&str; 1] = ["images"];
const DOCKER_FORMAT_ARGS: [&str; 2] = ["--format", "{{json .}}"];
const DOCKER_RMI_CMD: [&str; 1] = ["rmi"];
const DAYS_RM: u32 = 2;

/// Clear docker images from
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, takes_value = false)]
    verbose: bool,

    /// filter by repository name
    #[clap(short, long)]
    repository: Option<String>,

    /// add tags exclusion
    #[clap(short, long)]
    tags: Option<Vec<String>>,

    /// image cleanup will not be triggered
    #[clap(long, takes_value = false)]
    dry_run: bool,

    /// should docker force image removal (it may create orphan images)
    #[clap(long, takes_value = false)]
    force: bool,
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

    let mut cmd = Command::new(DOCKER_BIN);
    cmd.args(DOCKER_IMGS_CMD);

    if let Some(repo) = args.repository {
        cmd.arg(repo);
    }

    cmd.args(DOCKER_FORMAT_ARGS);

    let stdout = match cmd.output() {
        Ok(o) => {
            if !o.status.success() {
                eprintln!("{}", std::str::from_utf8(&o.stderr).unwrap());
                return eprintln!("failed to retrieve docker images. Please checkout STDERR");
            }

            o.stdout
        }
        Err(e) => return eprintln!("docker command failed: {}", e),
    };

    let s_data = std::str::from_utf8(&stdout).unwrap();
    let mut images: Vec<&str> = s_data.split("\n").collect();
    // * remove last empty line
    images.remove(images.len() - 1);

    if images.len() == 0 {
        println!("No images found for current timestamp and/or repository");
        return;
    }

    let now = Utc::now();
    let past_date = get_past_date(now.year(), now.month(), now.day(), DAYS_RM).and_hms(1, 0, 0);
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
        cmd.args(DOCKER_RMI_CMD);

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

fn failed_convert_size(e: ParseFloatError) -> f32 {
    eprintln!("failed to convert \"String\" to \"f32\": {}", e);
    exit(1);
}
