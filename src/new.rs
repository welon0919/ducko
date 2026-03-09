mod asset;
mod error;

use std::{fs, path::Path, process::Command};

use anyhow::Context;
use log::{debug, trace};

const QUESTIONS: [&str; 5] = [
    "Enter the title of your blog: ",
    "Description: ",
    "Base url (Can be filled in later): ",
    "Author name (Optional): ",
    "Author email (Optional): ",
];
const DEFAULT_CONFIG: &str = include_str!("../skeleton/config.yaml");
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{
    config::{CONFIG_PATH, SiteConfig},
    new::{asset::Asset, error::InitError},
};

/// Helper function for prompting  for new site options
/// # Errors
/// Will return `Err` if:
/// 1. `rustyline` failed to init or readline
/// 2. The operation si canceled via Ctrl-C or Ctrl-D
fn ask_questions() -> Result<SiteConfig, InitError> {
    let mut rl = DefaultEditor::new()?;
    let mut config: SiteConfig = serde_yaml::from_str(DEFAULT_CONFIG).unwrap();
    debug!("Default config: {config:?}");
    for (i, question) in QUESTIONS.iter().enumerate() {
        let readline =
            rl.readline(question).map(|line| line.trim().to_string());
        match readline {
            Ok(line) => match i {
                0 => {
                    if !line.is_empty() {
                        config.title = line;
                    }
                }
                1 => {
                    if !line.is_empty() {
                        config.description = line;
                    }
                }
                2 => {
                    if !line.is_empty() {
                        config.base_url = line;
                    }
                }
                3 => {
                    if !line.is_empty() {
                        config.author.name = line;
                    }
                }
                4 => {
                    if !line.is_empty() {
                        config.author.email = Some(line);
                    }
                }
                _ => panic!("Shouldn't be here"),
            },
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                println!("Init cancelled");
                return Err(InitError::InitCancelled);
            }
            Err(err) => {
                eprintln!("Readline error: {err}",);
                return Err(err.into());
            }
        }
    }

    Ok(config)
}
/// Create a new static site
/// # Errors
/// Will return `Err` if `setup_skeleton_files` failed
///
/// # Panics
/// Will panic if it failed to load the default assets
pub fn new() -> anyhow::Result<()> {
    let config = setup_skeleton_files()?;
    let git_installed = is_git_installed();
    if git_installed {
        println!("Git detected. Initializing repository...");
        let _ = Command::new("git")
            .arg("init")
            .current_dir(&config.title)
            .status();
    }
    println!("\n✅ Project '{}' initialized successfully!", config.title);
    println!("Next steps:");
    println!("  cd {}", config.title);
    println!("  {} serve --watch", env!("CARGO_PKG_NAME"));
    if git_installed {
        println!(
            "Next steps: Link this to GitHub to start auto-deploying via Actions!"
        );
    } else {
        println!(
            "⚠️ Warning: Git is not detected, it is recommended to install git at https://git-scm.com"
        );
        println!(
            "After you install git, run: \ngit init\ngit add .\ngit commit -m \"first commit\""
        );
    }

    Ok(())
}
/// Set up the default skeleton files
/// # Errors
/// 1. The target directory already exist
/// 2. The config failed to serialize
/// 3. It lacks the permission to write to the output folder
fn setup_skeleton_files() -> anyhow::Result<SiteConfig> {
    let config = ask_questions()?;
    let target_dir = Path::new(&config.title);
    if target_dir.exists() {
        anyhow::bail!(
            "Target directory already exists: {}",
            target_dir.display()
        );
    }
    fs::create_dir(&config.title).context("Failed to create directory")?;
    let config_yaml =
        serde_yaml::to_string(&config).context("Failed to serialize config")?;
    let config_path = Path::new(&config.title).join(CONFIG_PATH);
    fs::write(config_path, config_yaml).context("Failed to write config")?;
    for file_path in Asset::iter() {
        let content = Asset::get(file_path.as_ref()).unwrap();

        let target_path = target_dir.join(file_path.as_ref());

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target_path, content.data.as_ref()).with_context(|| {
            format!("Failed to create file: {}", target_path.display())
        })?;
        trace!("Created file: {file_path}",);
    }
    Ok(config)
}

/// Check if git is installed
fn is_git_installed() -> bool {
    Command::new("git")
        .arg("--version")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
