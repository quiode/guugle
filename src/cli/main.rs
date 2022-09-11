use clap::{Parser, Subcommand};

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
    Start {},
    // search in the db for a value
    #[clap(about = "Searches the database for the keyword")]
    Search {
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
        Commands::Start {} => start(),
        Commands::Search {
            search_word,
            amount,
        } => search(search_word, *amount),
    }
}

fn start() {
    todo!()
}

fn search(search_word: &str, amount: u32) {
    todo!()
}
