mod asset;
mod error;

use std::{
    borrow::Cow,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::Context;
use log::{debug, trace};

const QUESTIONS: [&str; 5] = [
    "Enter the title of your blog: ",
    "Description: ",
    "Base url: ",
    "Author name (Optional): ",
    "Author email (Optional): ",
];
const DEFAULT_CONFIG: &str = include_str!("../skeleton/config.yaml");
use rustyline::{
    Completer, DefaultEditor, Editor, Helper, Hinter, Validator,
    error::ReadlineError, highlight::Highlighter, history::DefaultHistory,
};

use crate::{
    config::{CONFIG_PATH, SiteConfig},
    new::{asset::Asset, error::InitError},
};

#[derive(Completer, Hinter, Validator)]
struct QuestionsHelper;

impl Helper for QuestionsHelper {}

impl Highlighter for QuestionsHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.is_empty() {
            Cow::Owned(
                "\x1b[90mLeave empty to fill in later...\x1b[0m".to_string(),
            )
        } else {
            Cow::Borrowed(line)
        }
    }
}
fn ask_questions() -> Result<SiteConfig, InitError> {
    let mut rl = Editor::<QuestionsHelper, DefaultHistory>::new()?;
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
                eprintln!("Readline error: {}", err);
                return Err(err.into());
            }
        }
    }

    Ok(config)
}
pub fn new() -> anyhow::Result<()> {
    let config = ask_questions()?;
    let target_dir = Path::new(&config.title);
    if target_dir.exists() {
        anyhow::bail!("Target directory already exists: {:?}", target_dir);
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
            format!("Failed to create file: {:?}", target_path)
        })?;
        trace!("Created file: {}", file_path);
    }
    println!("\n✅ Project '{}' initialized successfully!", config.title);
    println!("Next steps:");
    println!("  cd {}", config.title);
    println!("  {} serve --watch", env!("CARGO_PKG_NAME"));

    Ok(())
}
