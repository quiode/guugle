use crate::{db_manager::Ranking, indexer::Visited};

struct RankedPage {
    rank: i32,
    page: String,
}

/// takes a search word and returns a list with all search results and their relevancy
fn rank_pages(search_word: &str) -> Vec<RankedPage> {
    /*
    TODO:
    1. get all database entries with include the search word
    2. for each database entry, create a rank
    3. sort list based on rank
    */

    let matches: ();
    todo!();
}

fn compute_rank(link_to_count: i32, link_from_count: i32, search_word_appearance: i32) -> i32 {
    link_from_count + link_to_count + search_word_appearance
}

#[cfg(test)]
mod tests {
    // TODO: create tests for every function

    use super::compute_rank;

    #[test]
    fn test_rank_computation_normal() {
        assert!(compute_rank(8, 10, 20) > compute_rank(1, 0, 2));
    }

    #[test]
    fn test_rank_computation_zero() {
        assert!(compute_rank(0, 0, 0) < compute_rank(999, 999, 999));
    }
}
