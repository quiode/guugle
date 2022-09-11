use crate::db_manager::{calculate_links_from, find, DatabaseConnection, Ranking};

pub struct RankedPage {
    rank: usize,
    page: Ranking,
}

/// takes a search word and returns a list with all search results and their relevancy
pub fn rank_pages(
    conn: &DatabaseConnection,
    search_word: &str,
) -> Result<Vec<RankedPage>, rusqlite::Error> {
    let matches = find(conn, search_word)?;

    let mut ranking = vec![];

    for single_match in matches {
        let link_to_count = single_match
            .links_to
            .clone()
            .unwrap_or("".to_string())
            .split(":::")
            .collect::<Vec<_>>()
            .len();

        let link_from_count = calculate_links_from(&conn, single_match.id)?;

        let search_word_appearance = compute_search_word_appearance(search_word, &single_match.url)
            + compute_search_word_appearance(
                search_word,
                &single_match.content.clone().unwrap_or("".to_string()),
            );

        let has_whole_word = single_match.url.find(search_word).is_some()
            || single_match
                .content
                .clone()
                .unwrap_or("".to_string())
                .find(search_word)
                .is_some();

        ranking.push(RankedPage {
            page: single_match,
            rank: compute_rank(
                link_to_count,
                link_from_count,
                search_word_appearance,
                has_whole_word,
            ),
        })
    }

    ranking.sort_by(|a, b| b.rank.cmp(&a.rank));
    Ok(ranking)
}

fn compute_rank(
    link_to_count: usize,
    link_from_count: usize,
    search_word_appearance: usize,
    has_whole_word: bool,
) -> usize {
    let mut res = link_from_count + link_to_count + search_word_appearance;

    if has_whole_word {
        res += 10
    }

    res
}

fn compute_search_word_appearance(search_word: &str, string: &str) -> usize {
    let search_word = search_word.to_lowercase();
    let string = string.to_lowercase();
    let parts = search_word.split(" ").collect::<Vec<_>>();

    let mut appearance = 0;

    for part in parts {
        let matches = string.matches(part).collect::<Vec<_>>();

        appearance += matches.len();
    }

    appearance
}

#[cfg(test)]
mod tests {
    // TODO: create tests for every function

    use super::*;
    use crate::db_manager::{
        create_default_tables,
        tests::{gen_random_path, gen_vals},
    };
    use std::fs;

    use super::{compute_rank, compute_search_word_appearance, rank_pages};

    #[test]
    fn test_rank_computation_normal() {
        assert!(compute_rank(8, 10, 20, true) > compute_rank(1, 0, 2, false));
    }

    #[test]
    fn test_rank_computation_zero() {
        assert!(compute_rank(0, 0, 0, false) < compute_rank(999, 999, 999, true));
    }

    #[test]
    fn appearance_counter() {
        let search_word = "where do we go?";
        let content = "We go to the church. There we live. I don't know where we should go.";

        assert_eq!(compute_search_word_appearance(search_word, content), 5);
    }

    #[test]
    fn rank_single_word() {
        let path = gen_random_path();
        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        gen_vals(&conn);

        let result = rank_pages(&conn, "team").unwrap();

        assert_eq!(result.len(), 3);

        fs::remove_file(path).unwrap();
        assert!(result
            .iter()
            .any(|res| res.page.url == "ep.ch" && res.rank == 3));

    }
}
