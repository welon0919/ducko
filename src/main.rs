use clap::{Parser, Subcommand};
use ducko::{add_page, build, new, serve};

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new ducko site
    New,
    /// Add a new page to your site
    Add {
        /// True if you want to create a page bundle
        name: Option<String>,
        #[arg(long)]
        index: bool,
    },
    /// Serve a Dev server
    Serve {
        /// Whether you want live update or not
        #[arg(long, short, default_value_t = true)]
        watch: bool,
    },
    /// Build the site into HTML
    Build,
}
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
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
        Commands::Add { name, index } => {
            add_page(name, index)?;
        }
    }
    Ok(())
}
