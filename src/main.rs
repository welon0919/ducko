use clap::{Parser, Subcommand};
use static_site_generator::{build, serve};

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
            serve().await?;
        }
    }
    Ok(())
}
