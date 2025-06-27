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

pub use {config::Config, responder::Responder, statement::Statement};

fn main() {
    println!("Hello, world!");
    let _config = config::Config::load(None).unwrap();
}
