pub mod config;
pub mod duckdb_index;
pub mod garbage_collection;
pub mod indexing;
pub mod query;
pub mod responder;
pub mod runtime_settings;
pub mod statement;
pub mod templates;
pub mod test_utils;
pub mod util;

pub use {
    config::Config,
    responder::Responder,
    statement::{Statement, Statements},
};

use {
    clap::{Parser, Subcommand},
    duckdb_index::DuckdbIndex,
    indexing::Index,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate configuration file
    GenerateConfig,
    /// Load in test statements
    Load,
    /// Run tests
    Query { query: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::GenerateConfig => {
            println!("Generating config file...");
            match config::Config::load(None) {
                Ok(config) => {
                    println!("Config file already exists {}", config.log_dir().display());
                } // save the default config
                Err(e) => {
                    println!("Config file does not exist (error: {e}), creating...");
                    config::Config::write_default_config().unwrap();
                }
            }
        }
        Commands::Load => {
            println!("Loading test statements...");

            let config = config::Config::load(None).unwrap();

            let statements: Vec<Statement> = std::include_str!("../test_data/test_statements.md")
                .lines()
                .map(|line| Statement::new(line.to_string()))
                .collect();
            let statements = Statements::from(statements);
            let ddi = DuckdbIndex::default();
            ddi.index_statements(statements, &config).await.unwrap();
        }
        Commands::Query { query } => {
            println!("Running query...");
            let config = config::Config::load(None).unwrap();
            let ddi = DuckdbIndex::default();
            let result = ddi.query_statements(&config, query).await.unwrap();
            println!("{result}");
        }
    }
}
