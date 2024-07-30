mod date;

use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, Utc};
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
    /// filter by date.
    ///
    /// Can filter by a minimum age $DATE or from $FROM|$TO (%Y-%m-%dT%H:%M:%S%Z) [default: $NOW - 2 days]
    #[clap(short, long)]
    date: Option<String>,

    /// filter by repository name
    #[clap(short, long)]
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

    let min: DateTime<Utc>;
    let mut max_opt: Option<DateTime<Utc>> = None;
    let now = Utc::now();
    if let Some(date) = args.date {
        if date.contains("|") {
            let dates: Vec<&str> = date.split("|").collect();
            let base_from = if dates[0].contains("T") {
                dates[0].to_string()
            } else {
                format!("{}T00:00:00", dates[0])
            };
            let base_to = if dates[1].contains("T") {
                dates[1].to_string()
            } else {
                format!("{}T00:00:00", dates[1])
            };

            let base_from = NaiveDateTime::parse_from_str(&base_from, "%FT%T").unwrap_or_else(|e| {
                eprintln!(
                    "failed to parse from date. Verify its format (example: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS): {}",
                    e
                );
                exit(1);
            });
            let base_to = NaiveDateTime::parse_from_str(&base_to, "%FT%T").unwrap_or_else(|e| {
                eprintln!(
                    "failed to parse to date. Verify its format (example: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS): {}",
                    e
                );
                exit(1);
            });
            min = DateTime::<Utc>::from_utc(base_from, Utc);
            max_opt = Some(DateTime::<Utc>::from_utc(base_to, Utc));
        } else {
            let formatted_date = if date.contains("T") {
                date
            } else {
                date + "T00:00:00"
            };

            let date = NaiveDateTime::parse_from_str(&formatted_date, "%FT%T")
                .unwrap_or_else(|e| {
                    eprintln!(
                    "failed to parse date. Verify its format (example: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS): {}",
                    e
                );
                    exit(1);
                });

            min = DateTime::<Utc>::from_utc(date, Utc);
        }
    } else {
        min = get_past_date(now.year(), now.month(), now.day(), 2).and_hms(1, 0, 0);
    };

    let tags = if let Some(t) = args.tags { t } else { vec![] };
    let mut ids = vec![];
    let mut saved_size: f32 = 0.0;

    for img in images {
        let image: Image = serde_json::from_str(img).unwrap();
        let del;
        if let Some(max) = max_opt {
            del = if image.created_at.timestamp() <= min.timestamp()
                && image.created_at.timestamp() >= max.timestamp()
            {
                true
            } else {
                false
            }
        } else {
            del = if image.created_at.timestamp() <= min.timestamp() {
                true
            } else {
                false
            }
        }

        if del {
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
