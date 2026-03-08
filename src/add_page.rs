use std::{fs, path::Path};

use anyhow::bail;
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::build::MARKDOWN_PATH;

pub fn add_page(
    page_name: Option<String>,
    is_page_bundle: bool,
) -> anyhow::Result<()> {
    let name = match page_name {
        Some(n) if !n.trim().is_empty() => n,
        _ => ask_for_page_name()?,
    };

    let base_path = Path::new(MARKDOWN_PATH);

    let now = chrono::Local::now().format("%Y-%m-%d").to_string();
    let default_content = format!(
        "---
title: \"{}\"
date: \"{}\"
template: \"post.html\"
---

# {}
",
        name, now, name
    );

    if is_page_bundle {
        let folder_path = base_path.join(&name);
        if folder_path.exists() {
            bail!("Bundle directory '{}' already exists", name);
        }

        fs::create_dir_all(&folder_path)?;
        let file_path = folder_path.join("index.md");
        fs::write(file_path, default_content)?;
        println!("✨ Created page bundle: {}/index.md", folder_path.display());
    } else {
        let file_name = if name.ends_with(".md") {
            name
        } else {
            format!("{}.md", name)
        };
        let file_path = base_path.join(&file_name);

        if file_path.exists() {
            bail!("File '{}' already exists", file_path.display());
        }

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(file_path, default_content)?;
        println!("Created single page: {}/{}", MARKDOWN_PATH, file_name);
    }

    Ok(())
}

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
            Err(err.into())
        }
    }
}
