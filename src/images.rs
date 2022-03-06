use chrono::DateTime;
use log::{error, warn};
use serde::{self, Deserialize, Deserializer};
use std::process::{exit, Command};

use crate::DateArgs;
use crate::DOCKER_BIN;

const GHCR_REPO: &str = "ghcr.io/datahearth/clear-docker-images";
const DOCKER_REPO: &str = "datahearth/clear-docker-images";

#[derive(Deserialize, Debug)]
struct Image {
    // image ID
    #[serde(rename = "ID")]
    id: String,
    // image repository
    #[serde(rename = "Repository")]
    repository: String,
    // image tag
    #[serde(rename = "Tag")]
    tag: String,
    // image creation date as UNIX timestamp
    #[serde(deserialize_with = "deserialize_creation_date", rename = "CreatedAt")]
    created_at: i64,
    // image size in MB
    #[serde(deserialize_with = "deserialize_size", rename = "Size")]
    size: f32,
}

pub fn deserialize_creation_date<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let date = String::deserialize(deserializer)?;

    // format => 2021-01-01 00:00:00 +0100 CET
    DateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S %z %Z")
        .map(|d| d.timestamp())
        .map_err(serde::de::Error::custom)
}

pub fn deserialize_size<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let size = String::deserialize(deserializer)?;

    if size.contains("KB") {
        size.replace("KB", "")
            .parse::<f32>()
            .map(|s| s / 1000 as f32)
            .map_err(serde::de::Error::custom)
    } else if size.contains("MB") {
        size.replace("MB", "")
            .parse::<f32>()
            .map_err(serde::de::Error::custom)
    } else if size.contains("GB") {
        size.replace("GB", "")
            .parse::<f32>()
            .map(|s| s * 1000 as f32)
            .map_err(serde::de::Error::custom)
    } else {
        Err(serde::de::Error::custom(format!(
            "Unknown size identification: {}",
            size,
        )))
    }
}

pub fn process_imgs(
    repository: Option<String>,
    tags: Vec<String>,
    timestamps: DateArgs,
) -> (Vec<String>, f32) {
    let mut ids = vec![];
    let mut saved_size = 0.0;

    for img in parse_imgs(repository) {
        let image: Image = serde_json::from_str(&img).unwrap();
        let del = timestamps
            .stop
            .map_or(timestamps.start > image.created_at, |stop| {
                println!(
                    "stop date set, valid: {}",
                    timestamps.start > image.created_at && stop < image.created_at
                );
                timestamps.start > image.created_at && stop < image.created_at
            });

        if del && (image.repository != GHCR_REPO && image.repository != DOCKER_REPO) {
            if !tags.contains(&image.tag) {
                ids.push(image.id);

                saved_size += image.size
            }
        }
    }

    return (ids, saved_size);
}

fn get_images(repo: Option<String>) -> Vec<u8> {
    let mut cmd = Command::new(DOCKER_BIN);
    cmd.arg("images");

    repo.map(|repo| cmd.arg(repo));

    cmd.args(["--format", "{{json .}}"]);

    match cmd.output() {
        Ok(o) => {
            if !o.status.success() {
                error!(
                    "{}",
                    std::str::from_utf8(&o.stderr).expect("failed to parse STDERR to UTF-8")
                );
                error!("failed to retrieve docker images. Please checkout STDERR");
                exit(1);
            }

            o.stdout
        }
        Err(e) => {
            error!("docker command failed: {}", e);
            exit(1);
        }
    }
}

fn parse_imgs(repository: Option<String>) -> Vec<String> {
    let stdout = get_images(repository);

    let output = String::from_utf8(stdout).unwrap_or_else(|e| {
        error!("failed to parse docker output: {}", e);
        exit(1);
    });
    let mut images: Vec<String> = output.lines().map(|s| s.to_string()).collect();
    // * remove last empty line
    images.remove(images.len() - 1);

    if images.len() == 0 {
        warn!("No images found for current timestamp and/or repository");
        exit(1);
    }

    return images;
}
