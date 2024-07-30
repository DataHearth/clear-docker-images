mod date;
mod images;

use clap::Parser;
use std::process::{Command, Stdio};

use crate::images::process_imgs;

const DOCKER_BIN: &str = "docker";

/// Clear docker images from
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// filter by date.
    ///
    /// Can filter by a minimum age $DATE or from $FROM|$TO (format example: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS) [default: $NOW - 2 days]
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

fn main() {
    let args = Args::parse();

    let tags = if let Some(t) = args.tags { t } else { vec![] };
    let (ids, saved_size) = process_imgs(args.repository, tags, args.date);

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
