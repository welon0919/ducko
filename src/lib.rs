mod build;
mod metadata;

use std::{fmt::Display, net::SocketAddr};

use axum::Router;
pub use build::build;
use log::{debug, info};
use tower_http::services::ServeDir;

use crate::build::OUTPUT_PATH;
const PORT: u16 = 3333;
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
