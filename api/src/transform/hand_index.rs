use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::config::constants::{rank_index, RANKS};

static NAME_TO_INDEX: Lazy<HashMap<String, usize>> = Lazy::new(build_name_to_index);
static INDEX_TO_NAME: Lazy<Vec<String>> = Lazy::new(build_index_to_name);

fn build_name_to_index() -> HashMap<String, usize> {
    let mut map = HashMap::with_capacity(169);
    for (idx, name) in build_index_to_name().into_iter().enumerate() {
        map.insert(name, idx);
    }
    map
}

fn build_index_to_name() -> Vec<String> {
    let mut names = vec![String::new(); 169];
    for row in 0..13 {
        for col in 0..13 {
            let idx = row * 13 + col;
            let r1 = RANKS[row];
            let r2 = RANKS[col];
            if row == col {
                names[idx] = format!("{r1}{r2}");
            } else if col > row {
                names[idx] = format!("{r1}{r2}s");
            } else {
                names[idx] = format!("{r2}{r1}o");
            }
        }
    }
    names
}

pub fn hand_name_to_index(name: &str) -> Option<usize> {
    NAME_TO_INDEX.get(name).copied()
}

pub fn index_to_hand_name(index: usize) -> Option<&'static str> {
    INDEX_TO_NAME.get(index).map(|s| s.as_str())
}

pub fn hand_name_to_index_computed(name: &str) -> Option<usize> {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() == 2 {
        let r = rank_index(chars[0]);
        Some(r * 13 + r)
    } else if chars.len() == 3 {
        let r1 = rank_index(chars[0]);
        let r2 = rank_index(chars[1]);
        let suited = chars[2] == 's';
        if suited {
            let row = r1.min(r2);
            let col = r1.max(r2);
            Some(row * 13 + col)
        } else {
            let row = r1.max(r2);
            let col = r1.min(r2);
            Some(row * 13 + col)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairs() {
        assert_eq!(hand_name_to_index("AA"), Some(0));
        assert_eq!(hand_name_to_index("KK"), Some(14));
        assert_eq!(hand_name_to_index("22"), Some(168));
    }

    #[test]
    fn test_suited() {
        assert_eq!(hand_name_to_index("AKs"), Some(1));
        assert_eq!(hand_name_to_index("AQs"), Some(2));
        assert_eq!(hand_name_to_index("KQs"), Some(15));
    }

    #[test]
    fn test_offsuit() {
        assert_eq!(hand_name_to_index("AKo"), Some(13));
        assert_eq!(hand_name_to_index("AQo"), Some(26));
        // 72o: row=12 (rank '2'), col=7 (rank '7') → 12*13+7 = 163
        assert_eq!(hand_name_to_index("72o"), Some(163));
        // A7o: row=7 (rank '7'), col=0 (rank 'A') → 7*13+0 = 91
        assert_eq!(hand_name_to_index("A7o"), Some(91));
    }

    #[test]
    fn test_roundtrip() {
        for i in 0..169 {
            let name = index_to_hand_name(i).unwrap();
            assert_eq!(hand_name_to_index(name), Some(i));
        }
    }

    #[test]
    fn test_computed_matches_lookup() {
        for i in 0..169 {
            let name = index_to_hand_name(i).unwrap();
            assert_eq!(hand_name_to_index_computed(name), Some(i));
        }
    }
}
