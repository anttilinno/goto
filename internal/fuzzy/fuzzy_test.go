package fuzzy

import (
	"testing"
)

func TestLevenshteinDistance(t *testing.T) {
	tests := []struct {
		s1, s2   string
		expected int
	}{
		// Identical strings
		{"", "", 0},
		{"a", "a", 0},
		{"hello", "hello", 0},

		// Empty strings
		{"", "abc", 3},
		{"abc", "", 3},

		// Single character changes
		{"cat", "bat", 1},
		{"cat", "car", 1},
		{"cat", "cats", 1},
		{"cats", "cat", 1},

		// Multiple changes
		{"kitten", "sitting", 3},
		{"Saturday", "Sunday", 3},

		// Case insensitive
		{"Hello", "hello", 0},
		{"DEV", "dev", 0},

		// Common typos (transpositions count as 2 in Levenshtein)
		{"dev", "dve", 2},       // e->v, v->e (transposition = 2 substitutions)
		{"projects", "prjects", 1}, // missing 'o'
		{"config", "conifg", 2}, // i->f, f->i (transposition)
	}

	for _, tt := range tests {
		t.Run(tt.s1+"_"+tt.s2, func(t *testing.T) {
			result := LevenshteinDistance(tt.s1, tt.s2)
			if result != tt.expected {
				t.Errorf("LevenshteinDistance(%q, %q) = %d, expected %d", tt.s1, tt.s2, result, tt.expected)
			}
		})
	}
}

func TestSimilarity(t *testing.T) {
	tests := []struct {
		s1, s2      string
		minExpected float64
		maxExpected float64
	}{
		// Exact matches
		{"hello", "hello", 1.0, 1.0},
		{"", "", 1.0, 1.0},

		// Case-insensitive exact matches
		{"Hello", "hello", 1.0, 1.0},
		{"DEV", "dev", 1.0, 1.0},

		// Slight differences (transposition = 2 edits, so similarity is lower)
		{"dev", "dve", 0.3, 0.4},         // 2 edits / 3 chars = 0.33 similarity
		{"projects", "prjects", 0.85, 0.9}, // 1 edit / 8 chars = 0.875 similarity

		// More different
		{"hello", "world", 0.0, 0.4},
		{"abc", "xyz", 0.0, 0.1},
	}

	for _, tt := range tests {
		t.Run(tt.s1+"_"+tt.s2, func(t *testing.T) {
			result := Similarity(tt.s1, tt.s2)
			if result < tt.minExpected || result > tt.maxExpected {
				t.Errorf("Similarity(%q, %q) = %f, expected between %f and %f", tt.s1, tt.s2, result, tt.minExpected, tt.maxExpected)
			}
		})
	}
}

func TestIsSubstring(t *testing.T) {
	tests := []struct {
		query, target string
		expected      bool
	}{
		{"proj", "projects", true},
		{"proj", "myproject", true},
		{"dev", "dev", true},
		{"dev", "development", true},
		{"dev", "devops", true},
		{"DEV", "development", true}, // Case insensitive
		{"PROJ", "projects", true},
		{"xyz", "projects", false},
		{"projects", "proj", false}, // Query longer than target
		{"", "projects", true},      // Empty query is substring of anything
	}

	for _, tt := range tests {
		t.Run(tt.query+"_in_"+tt.target, func(t *testing.T) {
			result := IsSubstring(tt.query, tt.target)
			if result != tt.expected {
				t.Errorf("IsSubstring(%q, %q) = %v, expected %v", tt.query, tt.target, result, tt.expected)
			}
		})
	}
}

func TestFindSimilar(t *testing.T) {
	candidates := []string{"dev", "development", "projects", "project-x", "docs", "devops", "test"}

	tests := []struct {
		name         string
		query        string
		threshold    float64
		expectFirst  string   // Expected first result
		expectSubset []string // All results should include these
		expectNone   bool     // True if no results expected
	}{
		{
			name:        "typo dve -> dev",
			query:       "dve",
			threshold:   0.3, // Transposition gives ~0.33 similarity
			expectFirst: "dev",
		},
		{
			name:         "substring proj",
			query:        "proj",
			threshold:    0.5,
			expectSubset: []string{"projects", "project-x"},
		},
		{
			name:        "substring dev",
			query:       "dev",
			threshold:   0.5,
			expectFirst: "dev", // Exact match should be first
		},
		{
			name:       "no match with high threshold",
			query:      "xyz",
			threshold:  0.8,
			expectNone: true,
		},
		{
			name:         "close match prj -> projects",
			query:        "prj",
			threshold:    0.3,
			expectSubset: []string{"projects"}, // prj substring of projects gives boost
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			results := FindSimilar(tt.query, candidates, tt.threshold)

			if tt.expectNone {
				if len(results) > 0 {
					t.Errorf("Expected no results for %q, got %v", tt.query, results)
				}
				return
			}

			if len(results) == 0 {
				t.Errorf("Expected results for %q with threshold %f, got none", tt.query, tt.threshold)
				return
			}

			if tt.expectFirst != "" && results[0].Value != tt.expectFirst {
				t.Errorf("Expected first result to be %q, got %q (all results: %v)", tt.expectFirst, results[0].Value, results)
			}

			for _, expected := range tt.expectSubset {
				found := false
				for _, match := range results {
					if match.Value == expected {
						found = true
						break
					}
				}
				if !found {
					t.Errorf("Expected %q to be in results for %q, but it wasn't. Results: %v", expected, tt.query, results)
				}
			}
		})
	}
}

func TestFindSimilarNames(t *testing.T) {
	candidates := []string{"dev", "projects", "docs"}

	results := FindSimilarNames("dve", candidates, 0.3) // Lower threshold for transposition
	if len(results) == 0 {
		t.Error("Expected results for 'dve', got none")
		return
	}
	if results[0] != "dev" {
		t.Errorf("Expected first result to be 'dev', got %q", results[0])
	}
}

func TestFindSimilar_Deduplication(t *testing.T) {
	// Test that duplicates in candidates are handled
	candidates := []string{"dev", "dev", "development", "dev"}

	results := FindSimilar("dve", candidates, 0.6)

	// Count how many times "dev" appears
	devCount := 0
	for _, r := range results {
		if r.Value == "dev" {
			devCount++
		}
	}

	if devCount > 1 {
		t.Errorf("Expected 'dev' to appear at most once, but it appeared %d times", devCount)
	}
}

func TestFindSimilar_Sorting(t *testing.T) {
	candidates := []string{"abc", "ab", "a"}

	results := FindSimilar("abc", candidates, 0.0)

	// First result should be exact match
	if len(results) > 0 && results[0].Value != "abc" {
		t.Errorf("Expected exact match 'abc' to be first, got %v", results)
	}

	// Check results are sorted by similarity (descending)
	for i := 1; i < len(results); i++ {
		if results[i].Similarity > results[i-1].Similarity {
			t.Errorf("Results not sorted by similarity: %v", results)
			break
		}
	}
}

func TestLevenshteinDistanceEdgeCases(t *testing.T) {
	tests := []struct {
		s1, s2   string
		expected int
	}{
		// Empty strings
		{"", "", 0},
		{"", "a", 1},
		{"a", "", 1},
		{"", "abc", 3},
		{"abc", "", 3},

		// Single characters
		{"a", "a", 0},
		{"a", "b", 1},
		{"a", "A", 0}, // case insensitive

		// Common typos for directory aliases
		{"dev", "dve", 2},           // transposition
		{"projects", "prjects", 1},  // missing 'o'
		{"projects", "projcets", 2}, // transposition
		{"config", "conifg", 2},     // transposition

		// Prefix matching scenarios
		{"dev", "development", 8},
		{"proj", "projects", 4},

		// Completely different strings
		{"abc", "xyz", 3},
		{"hello", "world", 4},
	}

	for _, tt := range tests {
		t.Run(tt.s1+"_vs_"+tt.s2, func(t *testing.T) {
			result := LevenshteinDistance(tt.s1, tt.s2)
			if result != tt.expected {
				t.Errorf("LevenshteinDistance(%q, %q) = %d, expected %d", tt.s1, tt.s2, result, tt.expected)
			}
		})
	}
}

func TestFuzzySuggestions(t *testing.T) {
	// Common alias names
	candidates := []string{
		"dev",
		"development",
		"projects",
		"project-web",
		"project-api",
		"docs",
		"downloads",
		"desktop",
		"config",
		"configs",
	}

	tests := []struct {
		name         string
		query        string
		threshold    float64
		expectInTop3 []string
	}{
		{
			name:         "typo dve -> dev",
			query:        "dve",
			threshold:    0.3,
			expectInTop3: []string{"dev"},
		},
		{
			name:         "substring proj",
			query:        "proj",
			threshold:    0.5,
			expectInTop3: []string{"projects"},
		},
		{
			name:         "prefix dev",
			query:        "dev",
			threshold:    0.5,
			expectInTop3: []string{"dev", "development"},
		},
		{
			name:         "typo projcts",
			query:        "projcts",
			threshold:    0.5,
			expectInTop3: []string{"projects"},
		},
		{
			name:         "typo downlods",
			query:        "downlods",
			threshold:    0.5,
			expectInTop3: []string{"downloads"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			results := FindSimilar(tt.query, candidates, tt.threshold)

			if len(results) == 0 {
				t.Errorf("Expected suggestions for %q, got none", tt.query)
				return
			}

			// Check that expected results are in top 3
			top3 := make(map[string]bool)
			limit := 3
			if len(results) < limit {
				limit = len(results)
			}
			for i := 0; i < limit; i++ {
				top3[results[i].Value] = true
			}

			for _, expected := range tt.expectInTop3 {
				if !top3[expected] {
					t.Errorf("Expected %q in top 3 results for %q, got %v", expected, tt.query, results[:limit])
				}
			}
		})
	}
}

func TestSimilarityRange(t *testing.T) {
	// Ensure similarity is always in [0, 1]
	testPairs := []struct {
		s1, s2 string
	}{
		{"", ""},
		{"a", "a"},
		{"abc", "xyz"},
		{"hello", "hello"},
		{"short", "verylongstring"},
		{"verylongstring", "short"},
	}

	for _, tt := range testPairs {
		t.Run(tt.s1+"_"+tt.s2, func(t *testing.T) {
			sim := Similarity(tt.s1, tt.s2)
			if sim < 0.0 || sim > 1.0 {
				t.Errorf("Similarity(%q, %q) = %f, expected in range [0, 1]", tt.s1, tt.s2, sim)
			}
		})
	}
}

func TestIsSubstringEdgeCases(t *testing.T) {
	tests := []struct {
		query, target string
		expected      bool
	}{
		// Empty query is substring of everything
		{"", "", true},
		{"", "abc", true},

		// Target shorter than query
		{"abc", "ab", false},
		{"long", "lo", false},

		// Case variations
		{"ABC", "abcdef", true},
		{"abc", "ABCDEF", true},
		{"aBc", "xAbCy", true},

		// Middle of string
		{"bc", "abcd", true},
		{"xyz", "abcxyzdef", true},

		// Not a substring
		{"xyz", "abc", false},
		{"aaa", "aa", false},
	}

	for _, tt := range tests {
		t.Run(tt.query+"_in_"+tt.target, func(t *testing.T) {
			result := IsSubstring(tt.query, tt.target)
			if result != tt.expected {
				t.Errorf("IsSubstring(%q, %q) = %v, expected %v", tt.query, tt.target, result, tt.expected)
			}
		})
	}
}

func TestFindSimilar_EmptyInputs(t *testing.T) {
	// Empty candidates
	results := FindSimilar("query", []string{}, 0.5)
	if len(results) != 0 {
		t.Errorf("Expected no results for empty candidates, got %v", results)
	}

	// Empty query
	candidates := []string{"a", "b", "c"}
	results = FindSimilar("", candidates, 0.0)
	// Empty string is substring of everything, so all should match
	if len(results) != 3 {
		t.Errorf("Expected 3 results for empty query, got %d", len(results))
	}
}

func TestFindSimilar_ThresholdBoundary(t *testing.T) {
	candidates := []string{"abc"}

	// Exact match should always pass any threshold
	results := FindSimilar("abc", candidates, 1.0)
	if len(results) != 1 || results[0].Value != "abc" {
		t.Errorf("Exact match should pass threshold 1.0, got %v", results)
	}

	// Threshold 0 should accept everything
	results = FindSimilar("xyz", candidates, 0.0)
	if len(results) != 1 {
		t.Errorf("Threshold 0.0 should accept any match, got %v", results)
	}

	// High threshold should reject non-matches
	results = FindSimilar("xyz", candidates, 0.9)
	if len(results) != 0 {
		t.Errorf("High threshold should reject dissimilar strings, got %v", results)
	}
}
