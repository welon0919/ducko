use clap::{Parser, Subcommand};
use static_site_generator::{build, serve, watch_files};

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Build,
    Serve,
}
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    match args.command {
        Commands::Build => {
            build()?;
        }
        Commands::Serve => {
            std::thread::spawn(|| {
                if let Err(e) = watch_files() {
                    eprintln!("Error while watching files: {}", e);
                }
            });
            serve().await?;
        }
    }
    Ok(())
}
