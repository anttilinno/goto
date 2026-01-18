// Package fuzzy provides fuzzy string matching algorithms.
package fuzzy

import (
	"sort"
	"strings"
)

// Match represents a fuzzy match result with its similarity score.
type Match struct {
	Value      string
	Similarity float64
}

// LevenshteinDistance calculates the minimum number of single-character edits
// (insertions, deletions, or substitutions) required to change s1 into s2.
func LevenshteinDistance(s1, s2 string) int {
	s1 = strings.ToLower(s1)
	s2 = strings.ToLower(s2)

	if s1 == s2 {
		return 0
	}
	if len(s1) == 0 {
		return len(s2)
	}
	if len(s2) == 0 {
		return len(s1)
	}

	// Create two rows for the dynamic programming approach
	// We only need the previous row and current row
	prev := make([]int, len(s2)+1)
	curr := make([]int, len(s2)+1)

	// Initialize the first row
	for j := 0; j <= len(s2); j++ {
		prev[j] = j
	}

	// Fill in the rest of the matrix
	for i := 1; i <= len(s1); i++ {
		curr[0] = i
		for j := 1; j <= len(s2); j++ {
			cost := 1
			if s1[i-1] == s2[j-1] {
				cost = 0
			}
			curr[j] = min(
				prev[j]+1,      // deletion
				curr[j-1]+1,    // insertion
				prev[j-1]+cost, // substitution
			)
		}
		// Swap rows
		prev, curr = curr, prev
	}

	return prev[len(s2)]
}

// Similarity returns a similarity score between 0.0 and 1.0.
// 1.0 means exact match, 0.0 means completely different.
func Similarity(s1, s2 string) float64 {
	if s1 == s2 {
		return 1.0
	}

	s1Lower := strings.ToLower(s1)
	s2Lower := strings.ToLower(s2)

	if s1Lower == s2Lower {
		return 1.0
	}

	maxLen := max(len(s1), len(s2))
	if maxLen == 0 {
		return 1.0 // Both empty strings are identical
	}

	distance := LevenshteinDistance(s1, s2)
	return 1.0 - float64(distance)/float64(maxLen)
}

// IsSubstring checks if query is a substring of target (case-insensitive).
func IsSubstring(query, target string) bool {
	return strings.Contains(strings.ToLower(target), strings.ToLower(query))
}

// FindSimilar finds strings similar to the query from a list of candidates.
// It returns matches with similarity >= threshold, sorted by similarity (highest first).
// Substring matches are also included with a boosted score.
func FindSimilar(query string, candidates []string, threshold float64) []Match {
	var matches []Match
	seen := make(map[string]bool)

	for _, candidate := range candidates {
		if seen[candidate] {
			continue
		}
		seen[candidate] = true

		sim := Similarity(query, candidate)

		// Boost score for substring matches
		if IsSubstring(query, candidate) {
			// Substring match gets boosted similarity
			// The boost is proportional to how much of the candidate the query covers
			substringBoost := float64(len(query)) / float64(len(candidate))
			sim = max(sim, 0.5+substringBoost*0.5)
		}

		if sim >= threshold {
			matches = append(matches, Match{
				Value:      candidate,
				Similarity: sim,
			})
		}
	}

	// Sort by similarity (highest first), then alphabetically for ties
	sort.Slice(matches, func(i, j int) bool {
		if matches[i].Similarity != matches[j].Similarity {
			return matches[i].Similarity > matches[j].Similarity
		}
		return matches[i].Value < matches[j].Value
	})

	return matches
}

// FindSimilarNames returns just the names of similar matches.
func FindSimilarNames(query string, candidates []string, threshold float64) []string {
	matches := FindSimilar(query, candidates, threshold)
	names := make([]string, len(matches))
	for i, m := range matches {
		names[i] = m.Value
	}
	return names
}
