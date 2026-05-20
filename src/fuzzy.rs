const SCORE_MATCH: i32 = 16;
const BONUS_BOUNDARY: i32 = 8; // matched right after _, -, /, ., space
const BONUS_CAMEL: i32 = 6; // matched on uppercase after lowercase
const BONUS_CONSECUTIVE: i32 = 4; // previous query char also matched
const BONUS_FIRST_CHAR: i32 = 8; // matched at position 0
const PENALTY_GAP_START: i32 = -3; // first char skipped
const PENALTY_GAP_EXTEND: i32 = -1; // each additional skipped char

pub fn score(query: &str, candidate: &str) -> Option<i32> {
    let mut q = query.chars();
    let mut current = q.next();

    let mut total: i32 = 0;
    let mut prev_char: Option<char> = None;

    let mut last_matched = false;

    for (i, ch) in candidate.chars().enumerate() {
        let Some(ch_current) = current else {
            break;
        };

        if ch == ch_current {
            total += SCORE_MATCH;

            if last_matched {
                total += BONUS_CONSECUTIVE;
            }

            if let Some(prev_ch) = prev_char {
                if is_separator(prev_ch) {
                    total += BONUS_BOUNDARY;
                }

                if prev_ch.is_lowercase() && ch.is_uppercase() {
                    total += BONUS_CAMEL;
                }
            } else {
                total += BONUS_FIRST_CHAR;
            }

            last_matched = true;
            current = q.next();
        } else {
            if i == 0 {
                total += PENALTY_GAP_START;
            } else {
                total += PENALTY_GAP_EXTEND;
            }
            last_matched = false;
        }

        prev_char = Some(ch);
    }

    if current.is_none() {
        return Some(total);
    }

    None
}

fn is_separator(ch: char) -> bool {
    matches!(ch, '_' | '-' | '/' | '.' | ' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(q: &str, c: &str) -> i32 {
        score(q, c).unwrap_or(i32::MIN)
    }

    #[test]
    fn no_match_returns_none() {
        assert_eq!(score("xyz", "abc"), None);
    }

    #[test]
    fn prefix_beats_middle() {
        assert!(s("abc", "abc.txt") > s("abc", "xabc.txt"));
    }

    #[test]
    fn boundary_beats_random_position() {
        assert!(s("abc", "my_abc_file") > s("abc", "myabcfile"));
    }

    #[test]
    fn consecutive_beats_scattered() {
        assert!(s("abc", "abc.rs") > s("abc", "axbxc"));
    }

    #[test]
    fn camelcase_recognized() {
        assert!(s("GW", "GreatestWizard") > s("GW", "great_wizard"));
    }

    #[test]
    fn first_char_match_exact_score() {
        // SCORE_MATCH (16) + BONUS_FIRST_CHAR (8) = 24
        // If you get 32, you're double-counting the first-char bonus
        assert_eq!(score("a", "abc"), Some(24));
    }

    #[test]
    fn perfect_prefix_exact_score() {
        // pos 0: 16 + 8 (first)        = 24
        // pos 1: 16 + 4 (consecutive)  = 20
        // pos 2: 16 + 4 (consecutive)  = 20
        // total                        = 64
        assert_eq!(score("abc", "abcdef"), Some(64));
    }

    #[test]
    fn empty_query_matches_with_zero_score() {
        // empty query is trivially a subsequence of anything
        assert_eq!(score("", "anything"), Some(0));
    }

    #[test]
    fn empty_candidate_no_match() {
        assert_eq!(score("foo", ""), None);
    }

    #[test]
    fn query_longer_than_candidate_no_match() {
        assert_eq!(score("foobar", "foo"), None);
    }

    #[test]
    fn unicode_doesnt_panic() {
        // .chars() correctly handles multi-byte UTF-8
        assert!(score("é", "café").is_some());
        assert!(score("🦀", "rust 🦀 wins").is_some());
    }

    #[test]
    fn case_sensitive_for_now() {
        // your impl uses ==, so 'A' != 'a'. Document this assumption.
        // (real fzf has smart-case; that's a v1 feature)
        assert_eq!(score("ABC", "abc"), None);
    }

    #[test]
    fn shorter_gap_beats_longer_gap() {
        // both match but tighter is better
        assert!(s("ac", "aXc") > s("ac", "aXXXXXc"));
    }

    #[test]
    fn earlier_match_beats_later_match() {
        // same character distribution, but matching at start beats matching at end
        assert!(s("a", "aXXX") > s("a", "XXXa"));
    }

    #[test]
    fn consecutive_helps_when_starting_position_is_equal() {
        // both start at pos 1 (after 'x'), neither has boundaries
        // only difference: one is consecutive, the other has gaps
        // 53 > 43
        assert!(s("abc", "xabcyy") > s("abc", "xaybyc"));
    }

    #[test]
    fn boundary_helps_when_starting_position_is_equal() {
        // both 'a' is at pos 2, but one is right after a separator
        // 20 > 12
        assert!(s("a", "x_a") > s("a", "xxa"));
    }

    #[test]
    fn shorter_candidate_beats_longer_with_same_match_density() {
        // shorter candidate has fewer total chars to gap over
        // (this is the real-world fzf behavior)
        assert!(s("ac", "ac") > s("ac", "abbbbbbbc"));
    }

    #[test]
    fn matched_at_first_position_beats_matched_inside() {
        assert!(s("abc", "abc.txt") > s("abc", "x.abc.txt"));
    }
}
