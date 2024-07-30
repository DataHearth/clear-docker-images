use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::{
    num::ParseFloatError,
    process::{exit, Command},
};

use crate::date;
use crate::DOCKER_BIN;

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

pub fn process_imgs(
    repository: Option<String>,
    tags: Vec<String>,
    date: Option<String>,
) -> (Vec<String>, f32) {
    let (date_from, date_to) = date::parse_date(date);
    let mut ids = vec![];
    let mut saved_size: f32 = 0.0;

    for img in parse_imgs(repository) {
        let image: Image = serde_json::from_str(&img).unwrap();
        let del = if let Some(max) = date_to {
            if image.created_at.timestamp() >= date_from.timestamp()
                && image.created_at.timestamp() <= max.timestamp()
            {
                true
            } else {
                false
            }
        } else {
            if image.created_at.timestamp() >= date_from.timestamp() {
                true
            } else {
                false
            }
        };

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

    return (ids, saved_size);
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

fn parse_imgs(repository: Option<String>) -> Vec<String> {
    let stdout = get_images(repository);

    let output = String::from_utf8(stdout).unwrap_or_else(|e| {
        eprintln!("failed to parse docker output: {}", e);
        exit(1);
    });
    let mut images: Vec<String> = output.lines().map(|s| s.to_string()).collect();
    // * remove last empty line
    images.remove(images.len() - 1);

    if images.len() == 0 {
        println!("No images found for current timestamp and/or repository");
        exit(1);
    }

    return images;
}

fn failed_convert_size(e: ParseFloatError) -> f32 {
    eprintln!("failed to convert \"String\" to \"f32\": {}", e);
    exit(1);
}
