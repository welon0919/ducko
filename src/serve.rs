use std::{net::SocketAddr, path::Path, sync::mpsc::channel};

use axum::Router;
use log::{error, info, trace};
use notify::{Event, RecursiveMode, Watcher};
use tower_http::services::ServeDir;

use crate::{PORT, build, build::OUTPUT_PATH};

pub async fn serve() -> anyhow::Result<()> {
    build()?;
    let serve_dir = ServeDir::new(OUTPUT_PATH);
    let app = Router::new().fallback_service(serve_dir);
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    info!("listening on http://{addr}",);
    webbrowser::open(format!("http://{}", addr).as_str()).unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn watch_files() -> notify::Result<()> {
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(
        move |res: notify::Result<Event>| match res {
            Ok(event) => {
                if event.kind.is_modify() || event.kind.is_create() {
                    tx.send(event).unwrap();
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        },
    )?;
    watcher.watch(Path::new("content"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new("templates"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new("static"), RecursiveMode::Recursive)?;
    info!("Watching file changes");
    for event in rx {
        for path in event.paths {
            trace!("Changes detected  at file {}", path.display());
            // TODO add rebuild logic here
            if let Err(e) = build::handle_file_change(&path) {
                error!("File changing error {}", e);
            }
        }
    }
    Ok(())
}
