mod error;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{LazyLock, atomic::Ordering},
};

use fs_extra::dir::{CopyOptions, copy};
use log::{info, trace};
use pulldown_cmark::{CowStr, Event, Event::Text, Options, Parser, Tag, html};
use tera::Tera;

use crate::{
    build::error::BuildError, metadata::PostMetadata as Metadata,
    serve::WATCH_ENABLED,
};

pub const MARKDOWN_PATH: &str = "content";
pub const STATIC_PATH: &str = "static";
pub const OUTPUT_PATH: &str = "public";
const LIVE_RELOAD_SCRIPT: &str = r#"
            <script>
                const socket = new WebSocket('ws://' + window.location.host + '/livereload');
                socket.onmessage = (event) => {
                    if (event.data === 'RELOAD') {
                        window.location.reload();
                    }
                };
                socket.onclose = () => console.log('LiveReload connection closed.');
            </script>
        "#;
static OPTIONS: LazyLock<Options> = LazyLock::new(|| {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options
});
static TEMPLATE: LazyLock<Tera> =
    LazyLock::new(|| Tera::new("templates/**/*.html").unwrap());
pub fn build() -> Result<(), BuildError> {
    let path = PathBuf::from(MARKDOWN_PATH);
    trace!("Clearing directory {OUTPUT_PATH}");
    fs::remove_dir_all(OUTPUT_PATH)?;
    // build the content folder
    build_folder(&path)?;
    // build the  static folder
    build_static_folder()?;
    Ok(())
}
fn build_static_folder() -> Result<(), BuildError> {
    let mut options = CopyOptions::new();
    options.copy_inside = true;
    let static_path = PathBuf::from(STATIC_PATH);
    if static_path.exists() {
        copy(
            static_path,
            PathBuf::from(OUTPUT_PATH).join(STATIC_PATH),
            &options,
        )?;
    }
    Ok(())
}
fn build_folder(path: &Path) -> Result<(), BuildError> {
    let items = fs::read_dir(path).map_err(|e| {
        if let std::io::ErrorKind::NotFound = e.kind() {
            BuildError::ContentNotFound
        } else {
            e.into()
        }
    })?;
    let hash_index_md = path.join("index.md").exists();
    for item in items {
        let path = item?.path();
        if path.is_file() {
            if let Some(extension) =
                path.extension().and_then(|ext| ext.to_str())
            {
                match extension {
                    "md" => {
                        if let Err(e) = build_markdown_file(&path) {
                            return Err(BuildError::ErrorBuildingFile(
                                path.display().to_string(),
                                Box::new(e),
                            ));
                        }
                    }
                    "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg"
                    | "tiff" => {
                        build_image(&path)?;
                    }
                    _ => trace!("Skipping {}", path.display()),
                }
            }
        } else if path.is_dir() {
            if path.join("index.md").exists() {
                // the page is a page bundle
                trace!("Building page bundle {}", path.display());
                build_page_bundle(&path)?;
            } else {
                trace!("Building folder {}", path.display());
                build_page_bundle(&path)?;
                build_folder(&path)?;
            }
        }
    }
    Ok(())
}
fn build_image(path: &Path) -> Result<(), BuildError> {
    trace!("Building image {}", path.display());
    let output_path = get_output_file_folder(path)
        .parent()
        .unwrap()
        .join(path.file_name().unwrap());
    fs::create_dir_all(&output_path.parent().unwrap())?;
    fs::copy(path, &output_path)?;
    Ok(())
}
fn build_markdown_file(path: &Path) -> Result<(), BuildError> {
    trace!("Building markdown file {}", path.display());
    let (metadata, body_html) = parse_file(path)?;
    let html = apply_template(&TEMPLATE, body_html, metadata)?;
    trace!("Calling apply template");
    let is_root_index = path.file_name().map_or(false, |n| n == "index.md")
        && path.parent().map_or(false, |p| p.ends_with(MARKDOWN_PATH));
    let file_output_path = if is_root_index {
        PathBuf::from(OUTPUT_PATH).join("index.html")
    } else {
        get_output_file_folder(path).join("index.html")
    };
    trace!("Writing markdown to file {}", file_output_path.display());
    fs::create_dir_all(file_output_path.parent().unwrap())?;
    fs::write(&file_output_path, &html)?;
    Ok(())
}
fn apply_template(
    tera: &Tera,
    body_html: String,
    metadata: Metadata,
) -> Result<String, BuildError> {
    let mut context = tera::Context::new();
    context.insert("meta", &metadata);
    context.insert("content", &body_html);
    let template = metadata.template().unwrap_or("post.html");
    // TODO Insert global config here
    let mut html = tera
        .render("post.html", &context)
        .map_err(|e| BuildError::from(e))?;
    if WATCH_ENABLED.load(Ordering::Relaxed) {
        html.push_str(LIVE_RELOAD_SCRIPT);
    }
    Ok(html)
}
fn parse_file(path: &Path) -> Result<(Metadata, String), BuildError> {
    trace!("Parsing file {}", path.display());
    use BuildError as Error;
    let raw_content = fs::read_to_string(path)?;
    if !raw_content.starts_with("---\n") {
        return Err(Error::FrontMatterNotFound);
    }
    let parts: Vec<&str> = raw_content.splitn(3, "---").collect();

    if parts.len() < 3 {
        return Err(Error::FrontMatterNotClosed);
    }
    let yaml_str = parts[1];
    let markdown_str = parts[2];
    let metadata: Metadata = serde_yaml::from_str(yaml_str)?;
    let html_output = parse_html(markdown_str, "");

    Ok((metadata, html_output))
}
fn parse_html(markdown_str: &str, base_url: &str) -> String {
    let parser = Parser::new_ext(markdown_str, OPTIONS.clone());
    let transformed_events = parser.map(|event| match event {
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            if dest_url.starts_with("/") {
                let new_dest = format!("{}{}", base_url, dest_url);
                Event::Start(Tag::Image {
                    link_type,
                    dest_url: CowStr::from(new_dest),
                    title,
                    id,
                })
            } else {
                Event::Start(Tag::Image {
                    link_type,
                    dest_url,
                    title,
                    id,
                })
            }
        }
        _ => event,
    });
    let mut html_output = String::new();
    html::push_html(&mut html_output, transformed_events);
    html_output
}
fn get_output_file_folder(path: &Path) -> PathBuf {
    let rel_path = path
        .strip_prefix(MARKDOWN_PATH)
        .expect(format!("Path must be under {MARKDOWN_PATH}").as_str());

    let mut output = PathBuf::from(OUTPUT_PATH);

    if path.is_dir() {
        output.push(rel_path);
    } else {
        let parent = rel_path.parent().unwrap_or(Path::new(""));
        let stem = rel_path.file_stem().unwrap();
        if stem == "index" {
            // put index.html at root
            output.push(parent);
        } else {
            output.push(parent);
            output.push(stem);
        }
    }
    output
}
fn build_page_bundle(bundle_src_path: &Path) -> Result<(), BuildError> {
    let dest_dir = get_output_file_folder(bundle_src_path);
    fs::create_dir_all(&dest_dir)?;

    for entry in fs::read_dir(bundle_src_path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();

        if path.is_file() {
            let extension = path.extension().and_then(|s| s.to_str());

            match extension {
                Some("md") => {
                    trace!("Processing bundle content: {}", path.display());
                    build_markdown_file(&path)?;
                }
                Some(ext)
                    if matches!(
                        ext,
                        "png"
                            | "jpg"
                            | "jpeg"
                            | "gif"
                            | "bmp"
                            | "webp"
                            | "svg"
                            | "tiff"
                    ) =>
                {
                    let image_dest = dest_dir.join(file_name);
                    fs::copy(&path, image_dest)?;
                    trace!("Copied bundle image: {:?}", file_name);
                }
                _ => trace!("Skipping unknown bundle file: {:?}", file_name),
            }
        }
    }
    Ok(())
}
pub fn handle_file_change(absolute_path: &Path) -> Result<(), BuildError> {
    let current_dir = std::env::current_dir()?;
    let rel_path = absolute_path
        .strip_prefix(&current_dir)
        .unwrap_or(absolute_path);
    let path_str = rel_path.to_string_lossy();
    trace!("Handling file: {}", path_str);
    if path_str.contains("templates/") {
        info!("Template changed, performing full rebuild");
        build()?;
    } else if path_str.starts_with(MARKDOWN_PATH) {
        if let Some(ext) = rel_path.extension().and_then(|s| s.to_str()) {
            if ext == "md" {
                if let Some(parent) = rel_path.parent() {
                    if parent != Path::new(MARKDOWN_PATH) {
                        info!("Page bundle changed: {:?}", parent);
                        build_page_bundle(parent)?;
                    } else {
                        info!("Single markdown changed: {:?}", rel_path);
                        build_markdown_file(rel_path)?;
                    }
                }
            } else if matches!(
                ext,
                "png"
                    | "jpg"
                    | "jpeg"
                    | "gif"
                    | "bmp"
                    | "webp"
                    | "svg"
                    | "tiff"
            ) {
                if let Some(parent) = rel_path.parent() {
                    build_page_bundle(parent)?;
                }
            }
        }
    } else if path_str.starts_with(STATIC_PATH) {
        info!("Static asset changed, updating {}", rel_path.display());
        update_single_static_asset(rel_path)?;
    }
    Ok(())
}

fn update_single_static_asset(rel_path: &Path) -> Result<(), BuildError> {
    let dest_path = PathBuf::from(OUTPUT_PATH).join(rel_path);
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(rel_path, dest_path)?;
    Ok(())
}
