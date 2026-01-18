use std::cmp::min;

/// Match result with similarity score
#[derive(Debug, Clone)]
pub struct Match {
    pub value: String,
    pub similarity: f64,
}

/// Calculate Levenshtein distance between two strings (case-insensitive)
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1 = s1.to_lowercase();
    let s2 = s2.to_lowercase();

    if s1 == s2 {
        return 0;
    }
    if s1.is_empty() {
        return s2.len();
    }
    if s2.is_empty() {
        return s1.len();
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let mut prev: Vec<usize> = (0..=s2_chars.len()).collect();
    let mut curr = vec![0; s2_chars.len() + 1];

    for i in 1..=s1_chars.len() {
        curr[0] = i;
        for j in 1..=s2_chars.len() {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            curr[j] = min(
                min(prev[j] + 1, curr[j - 1] + 1),
                prev[j - 1] + cost,
            );
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[s2_chars.len()]
}

/// Calculate similarity score between 0.0 and 1.0
/// 1.0 = exact match, 0.0 = completely different
pub fn similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }

    let s1_lower = s1.to_lowercase();
    let s2_lower = s2.to_lowercase();

    if s1_lower == s2_lower {
        return 1.0;
    }

    let max_len = s1.len().max(s2.len());
    if max_len == 0 {
        return 1.0;
    }

    let distance = levenshtein_distance(s1, s2);
    1.0 - (distance as f64) / (max_len as f64)
}

/// Check if query is a substring of target (case-insensitive)
pub fn is_substring(query: &str, target: &str) -> bool {
    target.to_lowercase().contains(&query.to_lowercase())
}

/// Find strings similar to query from candidates
/// Returns matches with similarity >= threshold, sorted by similarity (highest first)
pub fn find_similar(query: &str, candidates: &[String], threshold: f64) -> Vec<Match> {
    let mut matches: Vec<Match> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for candidate in candidates {
        if seen.contains(candidate) {
            continue;
        }
        seen.insert(candidate.clone());

        let mut sim = similarity(query, candidate);

        // Boost score for substring matches
        if is_substring(query, candidate) {
            let substring_boost = query.len() as f64 / candidate.len() as f64;
            sim = sim.max(0.5 + substring_boost * 0.5);
        }

        if sim >= threshold {
            matches.push(Match {
                value: candidate.clone(),
                similarity: sim,
            });
        }
    }

    // Sort by similarity (highest first), then alphabetically
    matches.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.value.cmp(&b.value))
    });

    matches
}

/// Find similar names, returning just the string values
pub fn find_similar_names(query: &str, candidates: &[String], threshold: f64) -> Vec<String> {
    find_similar(query, candidates, threshold)
        .into_iter()
        .map(|m| m.value)
        .collect()
}

/// Find matches for a query among a list of candidates.
/// Returns matches sorted by score (highest first).
/// This function provides compatibility with code expecting the old interface.
pub fn find_matches<'a>(query: &str, candidates: impl Iterator<Item = &'a str>) -> Vec<(&'a str, i32)> {
    if query.is_empty() {
        // Return all candidates with score 0 for empty query
        return candidates.map(|c| (c, 0)).collect();
    }

    let mut matches: Vec<(&str, i32)> = Vec::new();

    for candidate in candidates {
        let sim = similarity(query, candidate);

        // Boost for substring matches
        let boosted_sim = if is_substring(query, candidate) {
            let substring_boost = query.len() as f64 / candidate.len() as f64;
            sim.max(0.5 + substring_boost * 0.5)
        } else {
            sim
        };

        // Convert similarity (0.0-1.0) to score (0-1000)
        // Only include if there's some match
        if boosted_sim >= 0.3 || is_substring(query, candidate) {
            let score = (boosted_sim * 1000.0) as i32;
            matches.push((candidate, score));
        }
    }

    // Sort by score descending, then by name ascending for ties
    matches.sort_by(|a, b| {
        b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0))
    });

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_one_edit() {
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        assert_eq!(levenshtein_distance("hello", "hellox"), 1);
    }

    #[test]
    fn test_similarity_exact() {
        assert_eq!(similarity("test", "test"), 1.0);
    }

    #[test]
    fn test_similarity_case_insensitive() {
        assert_eq!(similarity("Test", "test"), 1.0);
    }

    #[test]
    fn test_levenshtein_empty_strings() {
        assert_eq!(levenshtein_distance("", "hello"), 5);
        assert_eq!(levenshtein_distance("hello", ""), 5);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_levenshtein_case_insensitive() {
        assert_eq!(levenshtein_distance("Hello", "hello"), 0);
        assert_eq!(levenshtein_distance("HELLO", "hello"), 0);
    }

    #[test]
    fn test_similarity_bounds() {
        // Similarity should always be between 0.0 and 1.0
        let sim = similarity("abc", "xyz");
        assert!(sim >= 0.0 && sim <= 1.0);

        let sim = similarity("", "test");
        assert!(sim >= 0.0 && sim <= 1.0);
    }

    #[test]
    fn test_is_substring() {
        assert!(is_substring("ell", "Hello"));
        assert!(is_substring("ELL", "hello"));
        assert!(!is_substring("xyz", "hello"));
    }

    #[test]
    fn test_find_similar() {
        let candidates = vec![
            "projects".to_string(),
            "personal".to_string(),
            "work".to_string(),
        ];

        let matches = find_similar("proj", &candidates, 0.3);
        assert!(!matches.is_empty());
        // "projects" should match due to substring
        assert!(matches.iter().any(|m| m.value == "projects"));
    }

    #[test]
    fn test_find_similar_sorted_by_similarity() {
        let candidates = vec![
            "test".to_string(),
            "testing".to_string(),
            "tester".to_string(),
        ];

        let matches = find_similar("test", &candidates, 0.3);
        // Should be sorted by similarity (highest first)
        if matches.len() >= 2 {
            assert!(matches[0].similarity >= matches[1].similarity);
        }
    }

    #[test]
    fn test_find_similar_names() {
        let candidates = vec![
            "hello".to_string(),
            "world".to_string(),
        ];

        let names = find_similar_names("hell", &candidates, 0.3);
        assert!(names.contains(&"hello".to_string()));
    }

    #[test]
    fn test_find_similar_deduplicates() {
        let candidates = vec![
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
        ];

        let matches = find_similar("test", &candidates, 0.0);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_find_matches() {
        let candidates = vec!["projects", "personal", "prj", "work"];
        let matches = find_matches("pr", candidates.into_iter());

        assert!(!matches.is_empty());
        // Should match projects and prj (contain "pr")
        // "personal" doesn't contain "pr" and has low similarity so won't match
        assert!(matches.iter().any(|(name, _)| *name == "projects"));
        assert!(matches.iter().any(|(name, _)| *name == "prj"));
    }

    #[test]
    fn test_find_matches_empty_query() {
        let candidates = vec!["projects", "work"];
        let matches = find_matches("", candidates.into_iter());
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_find_matches_sorted_by_score() {
        let candidates = vec!["test", "testing", "tester"];
        let matches = find_matches("test", candidates.into_iter());

        // Should be sorted by score descending
        if matches.len() >= 2 {
            assert!(matches[0].1 >= matches[1].1);
        }
    }
}
