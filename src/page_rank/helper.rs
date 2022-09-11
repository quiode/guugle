pub fn compute_rank(
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

pub fn compute_search_word_appearance(search_word: &str, string: &str) -> usize {
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
    use crate::page_rank::helper::*;

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
}
