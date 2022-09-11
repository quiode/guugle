use regex::Regex;

use crate::db_manager::{
    creation::DatabaseConnection,
    ranking::Ranking,
    selecting::{calculate_links_from, find},
};

use super::helper::{compute_rank, compute_search_word_appearance};

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

        let regex = Regex::new(format!(r"(?i)\W+{search_word}\W+").as_str()).unwrap();
        let has_whole_word = regex.is_match(&single_match.url)
            || regex.is_match(&single_match.content.clone().unwrap_or("".to_string()));

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

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        db_manager::{
            creation::create_default_tables,
            helper::{gen_random_path, gen_vals},
        },
        page_rank::ranker::rank_pages,
    };

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

    #[test]
    fn rank_phrase() {
        let path = gen_random_path();
        let conn = create_default_tables(path.to_str().unwrap()).unwrap();

        gen_vals(&conn);

        let result = rank_pages(&conn, "anim tempor fugiat deserunt est").unwrap();

        fs::remove_file(path).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].page.url, "hre.he");
        assert_eq!(result[0].rank, 40);
    }
}
