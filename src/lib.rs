#[cfg(feature = "cli")]
pub mod cli;
mod db_manager;
mod indexer;
mod page_rank;
mod page_scraper;

pub use indexer::loops::run;
