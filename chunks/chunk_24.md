# Chunk 24: Fuzzy Matching

## Objective
When exact alias match fails, suggest similar aliases using fuzzy matching.

## Tasks

### 1. Implement Levenshtein Distance
```go
// internal/fuzzy/fuzzy.go

func LevenshteinDistance(s1, s2 string) int {
    // Calculate edit distance between strings
}

func Similarity(s1, s2 string) float64 {
    // Return 0.0-1.0 similarity score
    // 1.0 = exact match, 0.0 = completely different
}
```

### 2. Find Similar Aliases
```go
func (d *Database) FindSimilar(query string, threshold float64) []string {
    // Return aliases with similarity >= threshold
    // Sort by similarity (highest first)
}
```

### 3. Update Navigate Command
```go
func (c *Commands) Navigate(alias string) (string, error) {
    // Try exact match first
    path, err := c.db.Get(alias)
    if err == ErrNotFound {
        // Try fuzzy match
        suggestions := c.db.FindSimilar(alias, c.config.FuzzyThreshold)
        if len(suggestions) > 0 {
            return "", fmt.Errorf("alias '%s' not found. Did you mean: %s?",
                alias, strings.Join(suggestions, ", "))
        }
        return "", fmt.Errorf("alias '%s' not found", alias)
    }
    return path, nil
}
```

### 4. Configuration
```toml
[general]
fuzzy_threshold = 0.6  # 0.0-1.0, higher = stricter matching
```

### 5. Example Output
```
$ goto dve
Error: alias 'dve' not found. Did you mean: dev?

$ goto prj
Error: alias 'prj' not found. Did you mean: projects, project-x?
```

### 6. Substring Matching
Also match if query is a substring:
- `proj` matches `projects`, `myproject`
- `dev` matches `dev`, `development`, `devops`

## Files to Modify
- Create `internal/fuzzy/fuzzy.go` - Fuzzy matching algorithms
- `internal/database/database.go` - Add FindSimilar method
- `internal/commands/commands.go` - Update Navigate with suggestions

## Verification
- [ ] Exact match still works
- [ ] Fuzzy suggestions shown for typos
- [ ] Threshold from config is respected
- [ ] Multiple suggestions are shown
- [ ] Substring matching works
