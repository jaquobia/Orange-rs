use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct OrangeCliArgs {
    #[arg(short, long)]
    pub username: Option<String>,
    #[arg(short, long, value_name = "FILE")]
    pub orange_directory: Option<PathBuf>,
}
