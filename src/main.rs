use clap::{Parser, Subcommand};
use static_site_generator::{build, new, serve};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    Build,
    Serve {
        #[arg(long, short)]
        watch: bool,
    },
    New,
}
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    dbg!(&args);
    match args.command {
        Commands::Build => {
            build()?;
        }
        Commands::Serve { watch } => {
            serve(watch).await?;
        }
        Commands::New => {
            new()?;
        }
    }
    Ok(())
}
