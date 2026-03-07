use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, mpsc::channel},
    time::Duration,
};

use anyhow::Context;
use axum::{
    Router,
    extract::{WebSocketUpgrade, ws::Message},
    routing::get,
};
use log::{debug, error, info, trace};
use notify::{
    Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::ModifyKind,
};
use tokio::sync::{broadcast, broadcast::Sender};
use tower_http::services::ServeDir;

use crate::{
    PORT, build,
    build::{MARKDOWN_PATH, OUTPUT_PATH, STATIC_PATH, handle_file_change},
};
pub static WATCH_ENABLED: AtomicBool = AtomicBool::new(false);

pub async fn serve(watch_for_update: bool) -> anyhow::Result<()> {
    WATCH_ENABLED.store(watch_for_update, std::sync::atomic::Ordering::Relaxed);
    build().context("Initial build failed")?;
    let (reload_tx, _) = tokio::sync::broadcast::channel::<()>(16);
    if watch_for_update {
        let sender_tx = reload_tx.clone();
        std::thread::spawn(move || {
            if let Err(e) = watch(sender_tx) {
                error!("Watch thread died: {}", e);
            }
        });
    }
    let serve_dir = ServeDir::new(OUTPUT_PATH);
    let app = Router::new()
        .route(
            "/livereload",
            get({
                let tx = reload_tx.clone();
                move |ws| livereload_handler(ws, tx)
            }),
        )
        .fallback_service(serve_dir);
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    info!("listening on http://{addr}",);
    webbrowser::open(format!("http://{}", addr).as_str()).unwrap();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind listener")?;
    axum::serve(listener, app).await?;
    Ok(())
}
async fn livereload_handler(
    ws: WebSocketUpgrade,
    mut reload_tx: broadcast::Sender<()>,
) -> impl axum::response::IntoResponse {
    let mut reload_rx = reload_tx.subscribe();
    ws.on_upgrade(move |mut socket| async move {
        while let Ok(_) = reload_rx.recv().await {
            info!("reload");
            if socket.send(Message::Text("RELOAD".into())).await.is_err() {
                break;
            }
        }
    })
}

fn watch(reload_tx: tokio::sync::broadcast::Sender<()>) -> anyhow::Result<()> {
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    let output_abs = std::fs::canonicalize(OUTPUT_PATH)
        .unwrap_or_else(|_| PathBuf::from(OUTPUT_PATH));
    if Path::new(MARKDOWN_PATH).exists() {
        watcher.watch(Path::new(MARKDOWN_PATH), RecursiveMode::Recursive)?;
    }
    if Path::new(STATIC_PATH).exists() {
        watcher.watch(Path::new(STATIC_PATH), RecursiveMode::Recursive)?;
    }
    if Path::new("templates").exists() {
        watcher.watch(Path::new("templates"), RecursiveMode::Recursive)?;
    }
    info!("Listening for changes");
    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if matches!(event.kind, EventKind::Modify(_)) {
                    std::thread::sleep(Duration::from_millis(50));
                    // while let Ok(_) = rx.try_recv() {}
                    let mut needs_reload = false;
                    for path in event.paths {
                        let path_str = path.to_string_lossy();
                        if path_str.ends_with("~") || path_str.contains(".tmp")
                        {
                            trace!(
                                "Skipping temporary file: {}",
                                path.display()
                            );
                            continue;
                        }
                        if path.starts_with(&output_abs)
                            || path.to_string_lossy().contains(OUTPUT_PATH)
                        {
                            continue;
                        }
                        info!("File changed, handling {:?}", path);
                        if let Err(e) = handle_file_change(&path) {
                            error!("Handle file change error: {e}");
                        } else {
                            needs_reload = true;
                        }
                    }
                    if needs_reload {
                        debug!("Reloading file");
                        if let Err(e) = reload_tx.send(()) {
                            error!("Reload file send error: {e}");
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                error!("watch error: {e}");
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }
}
