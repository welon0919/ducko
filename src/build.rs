mod context;
mod error;

use std::{
    borrow::Cow,
    error::Error,
    fmt::Write as _,
    fs,
    path::{Component, Path, PathBuf},
    sync::{LazyLock, atomic::Ordering},
};

use fs_extra::dir::{CopyOptions, copy};
use log::{error, info, trace};
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};
use tera::Tera;

use crate::{
    build::{context::PostContext, error::BuildError},
    config::{CONFIG_PATH, SiteConfig},
    metadata::PostMetadata as Metadata,
    serve::WATCH_ENABLED,
};

pub const MARKDOWN_PATH: &str = "content";
pub const STATIC_PATH: &str = "static";
pub const OUTPUT_PATH: &str = "public";
const LIVE_RELOAD_SCRIPT: &str = r"
            <script>
                const socket = new WebSocket('ws://' + window.location.host + '/livereload');
                socket.onmessage = (event) => {
                    if (event.data === 'RELOAD') {
                        window.location.reload();
                    }
                };
                socket.onclose = () => console.log('LiveReload connection closed.');
            </script>
        ";
static OPTIONS: LazyLock<Options> = LazyLock::new(|| {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options
});

/// The entry point of the build process which builds from the foot
/// It also generates a sitemap
/// # Errors
/// Will return `Err` if the sub build functions failed, or config loading failed
pub fn build() -> Result<(), BuildError> {
    let path = PathBuf::from(MARKDOWN_PATH);
    if Path::new(OUTPUT_PATH).exists() {
        info!("Clearing directory {OUTPUT_PATH}");
        fs::remove_dir_all(OUTPUT_PATH)?;
    }
    trace!("Collecting all posts");
    let all_posts = collect_posts(&path)?;
    info!("Found {} posts", all_posts.len());
    let tera = load_templates();
    let site_config = SiteConfig::load_config()?;
    // build the content folder
    build_folder(&path, &all_posts, &tera, &site_config)?;
    let sitemap = generate_sitemap(&all_posts, &site_config);
    let sitemap_path = Path::new(OUTPUT_PATH).join("sitemap.xml");
    fs::write(sitemap_path, sitemap)?;
    // build the  static folder
    build_static_folder()?;
    Ok(())
}
/// Load the templates from the `templates` directory
fn load_templates() -> Tera {
    Tera::new("templates/**/*.html").unwrap()
}
/// Build the static folder
/// # Errors
/// Will return `Err` if it lacks permission to write to `OUTPUT_PATH`
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
/// Build the folder and keep recursions
/// # Errors
/// Will return `Err` if it lacks permission or any sub build commands return `Err`
fn build_folder(
    path: &Path,
    all_posts: &[PostContext],
    tera: &Tera,
    site_config: &SiteConfig,
) -> Result<(), BuildError> {
    let items = fs::read_dir(path).map_err(|e| {
        if let std::io::ErrorKind::NotFound = e.kind() {
            BuildError::ContentNotFound
        } else {
            e.into()
        }
    })?;
    for item in items {
        let path = item?.path();
        if path.is_file() {
            if let Some(extension) =
                path.extension().and_then(|ext| ext.to_str())
            {
                match extension {
                    "md" => {
                        if let Err(e) = build_markdown_file(
                            &path,
                            all_posts,
                            tera,
                            site_config,
                        ) {
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
                build_page_bundle(&path, all_posts, tera, site_config)?;
            } else {
                trace!("Building folder {}", path.display());
                build_page_bundle(&path, all_posts, tera, site_config)?;
                build_folder(&path, all_posts, tera, site_config)?;
            }
        }
    }
    Ok(())
}
/// Copy the image file to the target directory
/// # Errors
/// Will return `Err` if it lacks permission to write to the output directory
fn build_image(path: &Path) -> Result<(), BuildError> {
    trace!("Building image {}", path.display());
    let output_path = get_output_file_folder(path)
        .parent()
        .unwrap()
        .join(path.file_name().unwrap());
    fs::create_dir_all(output_path.parent().unwrap())?;
    fs::copy(path, &output_path)?;
    Ok(())
}
/// Build the Markdown file provided by `path`
/// # Errors
/// Will return `Err` if:
/// 1. Error happened when parsing Markdown file
/// 2. Error happened when applying template
/// 3. Lacks permission to write to output directory
fn build_markdown_file(
    path: &Path,
    all_posts: &[PostContext],
    tera: &Tera,
    config: &SiteConfig,
) -> Result<(), BuildError> {
    trace!("Building markdown file {}", path.display());
    let (metadata, body_html) = parse_file(path)?;
    let html =
        apply_template(tera, &body_html, &metadata, all_posts, config, path)?;
    trace!("Calling apply template");
    let should_be_put_at_root = path
        .file_name()
        .is_some_and(|n| n == "index.md" || n == "404.md")
        && path.parent().is_some_and(|p| p.ends_with(MARKDOWN_PATH));
    let file_output_path = if should_be_put_at_root {
        PathBuf::from(OUTPUT_PATH).join(format!(
            "{}.html",
            path.file_stem().unwrap().to_str().unwrap()
        ))
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
    body_html: &str,
    metadata: &Metadata,
    all_posts: &[PostContext],
    site_config: &SiteConfig,
    path: &Path,
) -> Result<String, BuildError> {
    let mut context = tera::Context::new();
    let url = path_to_url(path);
    context.insert("meta", &metadata);
    context.insert("content", &body_html);
    context.insert("posts", &all_posts);
    context.insert("site", &site_config);
    context.insert("current_url", &url);
    let template = if let Some(template) = metadata.template.as_deref() {
        template
    } else if path.file_name().is_some_and(|n| n == "index.md")
        && path.parent().is_some_and(|p| p.ends_with("content"))
    {
        "index.html"
    } else {
        "post.html"
    };
    let mut html = tera.render(template, &context).map_err(|e| {
        error!("Tera render error: {e}");
        let mut cause = e.source();
        while let Some(e) = cause {
            error!("Caused by: {e}");
            cause = e.source();
        }
        BuildError::from(e)
    })?;
    if WATCH_ENABLED.load(Ordering::Relaxed) {
        html.push_str(LIVE_RELOAD_SCRIPT);
    }
    Ok(html)
}
/// Parse the Markdown file into raw HTML String and `Metadata`
/// It doesn't apply template!
/// # Errors
/// Will return `Err` if the file is missing front matter or has incompatible fields
fn parse_file(path: &Path) -> Result<(Metadata, String), BuildError> {
    use BuildError as Error;
    trace!("Parsing file {}", path.display());
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
/// Parse the Markdown into HTML
fn parse_html(markdown_str: &str, base_url: &str) -> String {
    let parser = Parser::new_ext(markdown_str, *OPTIONS);
    let transformed_events = parser.map(|event| match event {
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            if dest_url.starts_with('/') {
                let new_dest = format!("{base_url}{dest_url}",);
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
/// Get the correct output file folder for the provided `path`
fn get_output_file_folder(path: &Path) -> PathBuf {
    let rel_path = path
        .strip_prefix(MARKDOWN_PATH)
        .unwrap_or_else(|_| panic!("Path must be under {MARKDOWN_PATH}"));

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
/// Build the directory if it's a page bundle
/// # Errors
/// Will return `Err` if:
/// 1. It lacks the permission to write the output folder
/// 2. The sub build function returned `Err`
fn build_page_bundle(
    bundle_src_path: &Path,
    all_posts: &[PostContext],
    tera: &Tera,
    site_config: &SiteConfig,
) -> Result<(), BuildError> {
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
                    build_markdown_file(&path, all_posts, tera, site_config)?;
                }
                Some(
                    "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg"
                    | "tiff",
                ) => {
                    let image_dest = dest_dir.join(file_name);
                    fs::copy(&path, image_dest)?;
                    trace!("Copied bundle image: {}", file_name.display());
                }
                _ => trace!(
                    "Skipping unknown bundle file: {}",
                    file_name.display()
                ),
            }
        }
    }
    Ok(())
}
/// Handle file change for `serve --watch`
/// Will return `Err` if the sub build functions failed
pub fn handle_file_change(absolute_path: &Path) -> Result<(), BuildError> {
    let all_posts = collect_posts(MARKDOWN_PATH)?;
    let tera = load_templates();
    let site_config = SiteConfig::load_config().unwrap();
    let current_dir = std::env::current_dir()?;
    let rel_path = absolute_path
        .strip_prefix(&current_dir)
        .unwrap_or(absolute_path);
    let path_str = rel_path.to_string_lossy();
    trace!("Handling file: {path_str}",);
    if path_str.contains(CONFIG_PATH) {
        info!("Config file changed, performing full rebuild");
        build()?;
    } else if path_str.contains("templates/") {
        info!("Template changed, performing full rebuild");
        build()?;
    } else if path_str.starts_with(MARKDOWN_PATH) {
        if let Some(ext) = rel_path.extension().and_then(|s| s.to_str()) {
            if ext == "md" {
                if let Some(parent) = rel_path.parent() {
                    if parent == Path::new(MARKDOWN_PATH) {
                        info!(
                            "Single markdown changed: {}",
                            rel_path.display()
                        );
                        build_markdown_file(
                            rel_path,
                            &all_posts,
                            &tera,
                            &site_config,
                        )?;
                    } else {
                        info!("Page bundle changed: {}", parent.display());
                        build_page_bundle(
                            parent,
                            &all_posts,
                            &tera,
                            &site_config,
                        )?;
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
            ) && let Some(parent) = rel_path.parent()
            {
                build_page_bundle(parent, &all_posts, &tera, &site_config)?;
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
/// Generate the `posts` template variable
/// # Errors
/// Will return `Err` if it lacks the permission to read the directory
fn collect_posts(
    path: impl AsRef<Path>,
) -> Result<Vec<PostContext>, BuildError> {
    let mut posts = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            posts.extend(collect_posts(&path)?);
        } else if let Some(ext) = path.extension()
            && ext == "md"
        {
            let (meta, _content) = parse_file(&path)?;

            let rel_path = path.strip_prefix(MARKDOWN_PATH).unwrap();
            let stem = rel_path.file_stem().unwrap().to_string_lossy();
            let parent = rel_path.parent().unwrap().to_string_lossy();

            let url = if stem == "index" && parent.is_empty() {
                "/".to_string()
            } else if stem == "index" {
                format!("/{parent}/",)
            } else {
                format!("/{parent}/{stem}/",)
            };

            posts.push(PostContext { meta, url });
        }
    }
    posts.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));
    Ok(posts)
}
#[must_use]
fn generate_sitemap(posts: &[PostContext], config: &SiteConfig) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#,
    );

    for post in posts {
        if post.url.contains("404") {
            continue;
        }
        let _ = write!(
            xml,
            "<url><loc>{}{}</loc><lastmod>{}</lastmod></url>",
            config.base_url, post.url, post.meta.date
        );
    }

    xml.push_str("</urlset>");
    xml
}

fn path_to_url(path: &Path) -> String {
    let mut components: Vec<_> = path
        .components()
        .filter_map(|c| {
            if let Component::Normal(s) = c {
                Some(s.to_string_lossy())
            } else {
                None
            }
        })
        .collect();

    if !components.is_empty() && components[0] == "content" {
        components.remove(0);
    }

    if let Some(last) = components.last_mut() {
        if last == "index.md" {
            components.pop();
        } else if last.ends_with(".md") {
            *last = Cow::from(last.trim_end_matches(".md").to_string());
        }
    }

    if components.is_empty() {
        "/".to_string()
    } else {
        format!("/{}/", components.join("/"))
    }
}
