mod error;
use std::{
    fs,
    fs::{DirEntry, Metadata},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use log::info;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

use crate::{build::error::BuildError, metadata::PostMetadata};

const MARKDOWN_PATH: &str = "content";
const OUTPUT_PATH: &str = "public";
static OPTIONS: LazyLock<Options> = LazyLock::new(|| {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options
});
pub fn build() -> Result<(), BuildError> {
    let path = PathBuf::from(MARKDOWN_PATH);
    build_folder(&path)
}
pub fn build_folder(path: &Path) -> Result<(), BuildError> {
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
        info!("Building {}", path.display());
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
                    _ => info!("Skipping {}", path.display()),
                }
            }
        } else if path.is_dir() {
            if path.join("index.md").exists() {
                // the page is a page bundle
                build_page_bundle(&path)?;
            } else {
                build_folder(&path)?;
            }
        }
    }
    Ok(())
}
fn build_image(path: &Path) -> Result<(), BuildError> {
    info!("Building image {}", path.display());
    let output_path = get_output_file_folder(path)
        .parent()
        .unwrap()
        .join(path.file_name().unwrap());
    fs::create_dir_all(&output_path.parent().unwrap())?;
    fs::copy(path, &output_path)?;
    Ok(())
}
fn build_markdown_file(path: &Path) -> Result<(), BuildError> {
    info!("Building markdown file {}", path.display());
    let (_metadata, html) = parse_file(path)?;
    // TODO add template logic
    // for now, we will not handle templates
    let file_output_path = if path == Path::new("content/index.md") {
        PathBuf::from(OUTPUT_PATH).join("index.html")
    } else {
        get_output_file_folder(path).join("index.html")
    };
    info!("Writing markdown to file {}", file_output_path.display());
    fs::create_dir_all(file_output_path.parent().unwrap())?;
    fs::write(&file_output_path, &html)?;
    Ok(())
}
fn parse_file(path: &Path) -> Result<(PostMetadata, String), BuildError> {
    info!("Parsing file {}", path.display());
    use BuildError as Error;
    let raw_content = std::fs::read_to_string(path)?;
    if !raw_content.starts_with("---\n") {
        return Err(Error::FrontMatterNotFound);
    }
    let parts: Vec<&str> = raw_content.splitn(3, "---").collect();

    if parts.len() < 3 {
        return Err(Error::FrontMatterNotClosed);
    }
    let yaml_str = parts[1];
    let markdown_str = parts[2];
    let metadata: PostMetadata = serde_yaml::from_str(yaml_str)?;
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
fn strip_to_dir<P: AsRef<Path>>(path: P, target: &str) -> Option<PathBuf> {
    let path = path.as_ref();
    let mut components = path.components().peekable();
    while let Some(component) = components.next() {
        if component.as_os_str() == target {
            let remaining: PathBuf = components.collect();
            return Some(remaining);
        }
    }
    None
}
fn get_output_file_folder(path: &Path) -> PathBuf {
    let rel_path = path
        .strip_prefix(MARKDOWN_PATH)
        .expect("路徑必須在 content 下");

    let mut output = PathBuf::from(OUTPUT_PATH);

    if path.is_dir() {
        output.push(rel_path);
    } else {
        let parent = rel_path.parent().unwrap_or(Path::new(""));
        let stem = rel_path.file_stem().unwrap();
        output.push(parent);
        output.push(stem);
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
                    info!("Processing bundle content: {}", path.display());
                    let (_metadata, html) = parse_file(&path)?;

                    let html_output_path = dest_dir.join("index.html");
                    fs::write(html_output_path, &html)?;
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
                    info!("Copied bundle image: {:?}", file_name);
                }
                _ => info!("Skipping unknown bundle file: {:?}", file_name),
            }
        }
    }
    Ok(())
}
