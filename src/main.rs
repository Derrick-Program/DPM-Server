mod action;
mod cli_parse;
mod json_parse;
mod zip_file;
pub use action::*;
use anyhow::Result;
use clap::Parser;
pub use cli_parse::*;
pub use json_parse::*;
use std::{
    collections::HashMap,
    env::current_dir,
    fs::{create_dir_all, File},
    io::Write,
};
pub use zip_file::*;
pub type Repos = HashMap<String, RepoInfo>;
#[derive(Parser)]
#[command(propagate_version = true)]
#[command(
    version,
    about,
    long_about = "Derrick Package Manager Server (DPM-Server)",
    styles = get_styles(),
)]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo_src = current_dir()?.join("Repo/src");
    let software_repo_info = current_dir()?.join("RepoInfo.json");
    create_dir_all(repo_src)?;
    if !software_repo_info.exists() {
        let mut file = File::create(software_repo_info)?;
        file.write_all(b"{}")?;
        repo_init()?;
    }
    match &cli.command {
        Commands::Hash(obj) => hash(obj)?,
        Commands::Fix(obj) => fix(obj)?,
        Commands::Build(obj) => build(obj)?,
        Commands::Init(obj) => init(obj)?,
    }
    Ok(())
}
