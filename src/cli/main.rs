use clap::{Parser, Subcommand};
use itertools::Itertools;

use crate::{db_manager::creation::create_default_tables, page_rank::ranker::rank_pages};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)] // Read from `Cargo.toml`
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // start the indexer
    #[clap(about = "Starts the indexer")]
    Start {
        #[clap(short, long, action, help = "output logs")]
        verbose: bool,
        #[clap(short, long, value_parser, help = "Sets the path for the database")]
        db_path: Option<String>,
        #[clap(
            short,
            long,
            value_parser,
            multiple_values = true,
            help = "Start values to start the indexing from",
            required = false
        )]
        start_values: Vec<String>,
        #[clap(value_parser = clap::value_parser!(u8), help = "Amount of threads to use", default_value_t = 8)]
        threads: u8,
    },
    // search in the db for a value
    #[clap(about = "Searches the database for the keyword")]
    Search {
        #[clap(short, long, action, help = "output logs")]
        verbose: bool,
        #[clap(short, long, value_parser, help = "Sets the path for the database")]
        db_path: Option<String>,
        #[clap(
            value_parser,
            help = "the key for which the database should be searched"
        )]
        search_word: String,
        #[clap(
            value_parser,
            default_value_t = 10,
            help = "the amount of results displayed"
        )]
        amount: u32,
    },
}

pub fn run() {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Start {
            verbose,
            db_path,
            start_values,
            threads,
        } => start(*verbose, db_path.clone(), start_values.to_vec(), *threads),
        Commands::Search {
            search_word,
            amount,
            verbose,
            db_path,
        } => search(search_word, *amount, *verbose, db_path.to_owned()),
    }
}

fn start(verbose: bool, db_path: Option<String>, start_urls: Vec<String>, threads: u8) {
    if verbose {
        println!("Starting Indexer...");
    }

    let start_urls = start_urls.iter().map(|x| x.as_str()).collect_vec();

    crate::run(start_urls, db_path, verbose, threads);

    if verbose {
        println!("Crawler finished");
    }
}

fn search(search_word: &str, amount: u32, verbose: bool, db_path: Option<String>) {
    if verbose {
        println!("Searchword: {search_word}, Amount: {amount}. Starting search....");
    }

    let db_path = db_path.unwrap_or("./database.db3".to_owned());

    let conn = create_default_tables(&db_path).unwrap();

    let results = rank_pages(&conn, search_word, amount).unwrap();

    for (i, result) in results.iter().enumerate() {
        println!("{i}. {:?}", result);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn verify_cli() {
        use super::Cli;
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
