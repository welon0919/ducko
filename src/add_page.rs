use std::{fs, path::Path};

use anyhow::{Context, bail};
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::build::MARKDOWN_PATH;

/// Add a page to the site
/// # Errors
/// It will return `Err` if:
/// 1. `ask_for_page_name` returned `Err`
/// 2. The page file / directory already exist
/// 3. It lacks the permission to write to the ouptut folder
pub fn add_page(
    page_name: Option<String>,
    is_page_bundle: bool,
) -> anyhow::Result<()> {
    let name = match page_name {
        Some(n) if !n.trim().is_empty() => n,
        _ => ask_for_page_name().context("Failed to ask for new page name")?,
    };

    let base_path = Path::new(MARKDOWN_PATH);

    let now = chrono::Local::now().format("%Y-%m-%d").to_string();
    let default_content = format!(
        "---
title: \"{name}\"
date: \"{now}\"
template: \"post.html\"
---

# {name}
",
    );

    if is_page_bundle {
        let folder_path = base_path.join(&name);
        if folder_path.exists() {
            bail!("Bundle directory '{name}' already exists",);
        }

        fs::create_dir_all(&folder_path)?;
        let file_path = folder_path.join("index.md");
        fs::write(file_path, default_content)?;
        println!("Created page bundle: {}/index.md", folder_path.display());
    } else {
        let file_name = if Path::new(&name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            name
        } else {
            format!("{name}.md",)
        };
        let file_path = base_path.join(&file_name);

        if file_path.exists() {
            bail!("File '{}' already exists", file_path.display());
        }

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(file_path, default_content)?;
        println!("Created single page: {MARKDOWN_PATH}/{file_name}",);
    }

    Ok(())
}

/// Ask for the name of the new page
/// # Errors
/// It will return `Err` if:
/// 1. `rustyline` failed to init
/// 2. The action is aborted
/// 3. The user did not enter a line
fn ask_for_page_name() -> Result<String, ReadlineError> {
    let mut rl = DefaultEditor::new()?;

    let readline = rl
        .readline("Enter name for your page")
        .map(|line| line.trim().to_string());
    match readline {
        Ok(line) => {
            if line.is_empty() {
                Err(ReadlineError::Eof)
            } else {
                Ok(line)
            }
        }
        Err(e @ (ReadlineError::Interrupted | ReadlineError::Eof)) => {
            println!("Init cancelled");
            Err(e)
        }
        Err(err) => {
            eprintln!("Readline error: {err}",);
            Err(err)
        }
    }
}
