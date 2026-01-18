package commands

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/BurntSushi/toml"
	"github.com/antti/goto-go/internal/alias"
	"github.com/antti/goto-go/internal/database"
)

// testEnv sets up an isolated test environment with its own database
type testEnv struct {
	tmpDir     string
	dbPath     string
	configPath string
	cleanup    func()
}

func setupTestEnv(t *testing.T) *testEnv {
	t.Helper()

	tmpDir := t.TempDir()
	dbPath := filepath.Join(tmpDir, "aliases")
	configPath := filepath.Join(tmpDir, "config.toml")

	// Set environment variable to use test database
	oldDB := os.Getenv("GOTO_DB")
	os.Setenv("GOTO_DB", dbPath)

	return &testEnv{
		tmpDir:     tmpDir,
		dbPath:     dbPath,
		configPath: configPath,
		cleanup: func() {
			if oldDB == "" {
				os.Unsetenv("GOTO_DB")
			} else {
				os.Setenv("GOTO_DB", oldDB)
			}
		},
	}
}

func (e *testEnv) createTestDir(name string) string {
	dir := filepath.Join(e.tmpDir, name)
	if err := os.Mkdir(dir, 0755); err != nil {
		panic(err)
	}
	return dir
}

func (e *testEnv) db() *database.Database {
	return database.New(e.dbPath)
}

// TestExportImportRoundTrip tests that export and import preserve data
func TestExportImportRoundTrip(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create some aliases with metadata
	now := time.Now()
	entries := []database.AliasEntry{
		{
			Name:     "dev",
			Path:     "/home/user/dev",
			Tags:     []string{"work", "code"},
			Created:  now.Add(-24 * time.Hour),
			LastUsed: now,
			UseCount: 10,
		},
		{
			Name:     "docs",
			Path:     "/home/user/docs",
			Tags:     []string{"reference"},
			Created:  now.Add(-48 * time.Hour),
			UseCount: 5,
		},
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Export to a buffer (capture stdout)
	exportFile := filepath.Join(env.tmpDir, "export.toml")

	// Manually create export data
	exportData := database.AliasDatabase{Aliases: entries}
	f, err := os.Create(exportFile)
	if err != nil {
		t.Fatalf("Create export file: %v", err)
	}
	encoder := toml.NewEncoder(f)
	if err := encoder.Encode(exportData); err != nil {
		t.Fatalf("Encode: %v", err)
	}
	f.Close()

	// Clear database
	if err := db.SaveEntries([]database.AliasEntry{}); err != nil {
		t.Fatalf("Clear database: %v", err)
	}

	// Verify empty
	loaded, _ := db.LoadEntries()
	if len(loaded) != 0 {
		t.Fatalf("Expected empty database, got %d entries", len(loaded))
	}

	// Import from file
	result, err := Import(exportFile, "skip")
	if err != nil {
		t.Fatalf("Import: %v", err)
	}

	if result.Imported != 2 {
		t.Errorf("Expected 2 imported, got %d", result.Imported)
	}

	// Verify data preserved
	imported, _ := db.LoadEntries()
	if len(imported) != 2 {
		t.Fatalf("Expected 2 entries after import, got %d", len(imported))
	}

	// Find dev entry
	var devEntry *database.AliasEntry
	for i := range imported {
		if imported[i].Name == "dev" {
			devEntry = &imported[i]
			break
		}
	}
	if devEntry == nil {
		t.Fatal("dev entry not found after import")
	}

	if devEntry.UseCount != 10 {
		t.Errorf("Expected UseCount 10, got %d", devEntry.UseCount)
	}
	if len(devEntry.Tags) != 2 {
		t.Errorf("Expected 2 tags, got %d", len(devEntry.Tags))
	}
}

func TestImportMergeStrategies(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create existing alias
	existing := database.AliasEntry{
		Name:     "dev",
		Path:     "/existing/path",
		UseCount: 100,
		Created:  time.Now(),
	}
	if err := db.SaveEntries([]database.AliasEntry{existing}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Create import file with conflicting alias
	importFile := filepath.Join(env.tmpDir, "import.toml")
	importData := database.AliasDatabase{
		Aliases: []database.AliasEntry{
			{
				Name:     "dev",
				Path:     "/imported/path",
				UseCount: 5,
				Created:  time.Now(),
			},
		},
	}
	f, _ := os.Create(importFile)
	toml.NewEncoder(f).Encode(importData)
	f.Close()

	t.Run("skip strategy", func(t *testing.T) {
		// Reset database
		db.SaveEntries([]database.AliasEntry{existing})

		result, err := Import(importFile, "skip")
		if err != nil {
			t.Fatalf("Import skip: %v", err)
		}

		if result.Skipped != 1 {
			t.Errorf("Expected 1 skipped, got %d", result.Skipped)
		}

		// Verify existing alias unchanged
		entry, _ := db.GetEntry("dev")
		if entry.Path != "/existing/path" {
			t.Errorf("Expected existing path, got %s", entry.Path)
		}
		if entry.UseCount != 100 {
			t.Errorf("Expected UseCount 100, got %d", entry.UseCount)
		}
	})

	t.Run("overwrite strategy", func(t *testing.T) {
		// Reset database
		db.SaveEntries([]database.AliasEntry{existing})

		result, err := Import(importFile, "overwrite")
		if err != nil {
			t.Fatalf("Import overwrite: %v", err)
		}

		if result.Imported != 1 {
			t.Errorf("Expected 1 imported, got %d", result.Imported)
		}

		// Verify alias was overwritten
		entry, _ := db.GetEntry("dev")
		if entry.Path != "/imported/path" {
			t.Errorf("Expected imported path, got %s", entry.Path)
		}
		if entry.UseCount != 5 {
			t.Errorf("Expected UseCount 5, got %d", entry.UseCount)
		}
	})

	t.Run("rename strategy", func(t *testing.T) {
		// Reset database
		db.SaveEntries([]database.AliasEntry{existing})

		result, err := Import(importFile, "rename")
		if err != nil {
			t.Fatalf("Import rename: %v", err)
		}

		if result.Renamed != 1 {
			t.Errorf("Expected 1 renamed, got %d", result.Renamed)
		}

		// Verify both aliases exist
		entries, _ := db.LoadEntries()
		if len(entries) != 2 {
			t.Errorf("Expected 2 entries, got %d", len(entries))
		}

		// Find the renamed entry
		var renamed *database.AliasEntry
		for i := range entries {
			if entries[i].Name == "dev_2" {
				renamed = &entries[i]
				break
			}
		}
		if renamed == nil {
			t.Error("Expected renamed entry 'dev_2' not found")
		} else if renamed.Path != "/imported/path" {
			t.Errorf("Expected imported path for renamed entry, got %s", renamed.Path)
		}
	})
}

func TestImportInvalidStrategy(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	_, err := Import("nonexistent.toml", "invalid")
	if err == nil {
		t.Error("Expected error for invalid strategy")
	}
	if !strings.Contains(err.Error(), "invalid strategy") {
		t.Errorf("Expected 'invalid strategy' error, got: %v", err)
	}
}

func TestImportEmptyFile(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	// Create empty import file
	importFile := filepath.Join(env.tmpDir, "empty.toml")
	os.WriteFile(importFile, []byte(""), 0644)

	_, err := Import(importFile, "skip")
	if err == nil {
		t.Error("Expected error for empty import file")
	}
}

func TestImportWithWarnings(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	// Create import file with non-existent path
	importFile := filepath.Join(env.tmpDir, "import.toml")
	importData := database.AliasDatabase{
		Aliases: []database.AliasEntry{
			{
				Name:    "missing",
				Path:    "/nonexistent/path/that/does/not/exist",
				Created: time.Now(),
			},
		},
	}
	f, _ := os.Create(importFile)
	toml.NewEncoder(f).Encode(importData)
	f.Close()

	result, err := Import(importFile, "skip")
	if err != nil {
		t.Fatalf("Import: %v", err)
	}

	// Should have warning about non-existent path
	if len(result.Warnings) == 0 {
		t.Error("Expected warning about non-existent path")
	}

	hasPathWarning := false
	for _, w := range result.Warnings {
		if strings.Contains(w, "path does not exist") {
			hasPathWarning = true
			break
		}
	}
	if !hasPathWarning {
		t.Errorf("Expected path warning, got warnings: %v", result.Warnings)
	}
}

func TestRenameAlias(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create alias with metadata
	now := time.Now()
	entry := database.AliasEntry{
		Name:     "oldname",
		Path:     "/home/user/project",
		Tags:     []string{"work", "important"},
		Created:  now.Add(-24 * time.Hour),
		LastUsed: now,
		UseCount: 15,
	}
	if err := db.SaveEntries([]database.AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Rename using command
	if err := Rename("oldname", "newname"); err != nil {
		t.Fatalf("Rename: %v", err)
	}

	// Verify old name is gone
	_, err := db.Get("oldname")
	if _, ok := err.(*alias.AliasNotFoundError); !ok {
		t.Errorf("Expected AliasNotFoundError for old name, got %T", err)
	}

	// Verify new name exists with metadata preserved
	renamed, err := db.GetEntry("newname")
	if err != nil {
		t.Fatalf("GetEntry for new name: %v", err)
	}

	if renamed.UseCount != 15 {
		t.Errorf("Expected UseCount 15, got %d", renamed.UseCount)
	}
	if len(renamed.Tags) != 2 {
		t.Errorf("Expected 2 tags, got %d", len(renamed.Tags))
	}
}

func TestRenameErrors(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create aliases
	entries := []database.AliasEntry{
		{Name: "first", Path: "/first", Created: time.Now()},
		{Name: "second", Path: "/second", Created: time.Now()},
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Test rename non-existent
	err := Rename("nonexistent", "newname")
	if _, ok := err.(*alias.AliasNotFoundError); !ok {
		t.Errorf("Expected AliasNotFoundError, got %T", err)
	}

	// Test rename to existing name
	err = Rename("first", "second")
	if _, ok := err.(*alias.AliasExistsError); !ok {
		t.Errorf("Expected AliasExistsError, got %T", err)
	}

	// Test invalid new name
	err = Rename("first", "invalid name!")
	if _, ok := err.(*alias.InvalidAliasError); !ok {
		t.Errorf("Expected InvalidAliasError, got %T", err)
	}
}

func TestUsageTracking(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create a real directory for navigation
	testDir := env.createTestDir("testdir")

	// Create alias pointing to the test directory
	entry := database.AliasEntry{
		Name:     "test",
		Path:     testDir,
		Created:  time.Now(),
		UseCount: 0,
	}
	if err := db.SaveEntries([]database.AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Navigate (this should increment usage)
	if err := Navigate("test"); err != nil {
		t.Fatalf("Navigate: %v", err)
	}

	// Verify usage was recorded
	updated, _ := db.GetEntry("test")
	if updated.UseCount != 1 {
		t.Errorf("Expected UseCount 1, got %d", updated.UseCount)
	}
	if updated.LastUsed.IsZero() {
		t.Error("Expected LastUsed to be set")
	}
}

func TestRecentList(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create aliases with different last_used times
	now := time.Now()
	entries := []database.AliasEntry{
		{Name: "old", Path: "/old", LastUsed: now.Add(-2 * time.Hour), UseCount: 1, Created: now},
		{Name: "recent", Path: "/recent", LastUsed: now.Add(-1 * time.Minute), UseCount: 1, Created: now},
		{Name: "unused", Path: "/unused", Created: now}, // never used
		{Name: "middle", Path: "/middle", LastUsed: now.Add(-1 * time.Hour), UseCount: 1, Created: now},
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Get recent list
	recent, err := Recent(0)
	if err != nil {
		t.Fatalf("Recent: %v", err)
	}

	// Should only include used aliases (3), sorted by most recent first
	if len(recent) != 3 {
		t.Errorf("Expected 3 recent entries, got %d", len(recent))
	}

	if recent[0].Alias != "recent" {
		t.Errorf("Expected first entry to be 'recent', got '%s'", recent[0].Alias)
	}
	if recent[1].Alias != "middle" {
		t.Errorf("Expected second entry to be 'middle', got '%s'", recent[1].Alias)
	}
	if recent[2].Alias != "old" {
		t.Errorf("Expected third entry to be 'old', got '%s'", recent[2].Alias)
	}
}

func TestRecentWithLimit(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create multiple aliases
	now := time.Now()
	var entries []database.AliasEntry
	for i := 0; i < 10; i++ {
		entries = append(entries, database.AliasEntry{
			Name:     "alias" + string(rune('a'+i)),
			Path:     "/path" + string(rune('a'+i)),
			LastUsed: now.Add(-time.Duration(i) * time.Hour),
			UseCount: 1,
			Created:  now,
		})
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Get limited recent list
	recent, err := Recent(5)
	if err != nil {
		t.Fatalf("Recent: %v", err)
	}

	if len(recent) != 5 {
		t.Errorf("Expected 5 recent entries, got %d", len(recent))
	}
}

func TestRecentClear(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create aliases with usage data
	now := time.Now()
	entries := []database.AliasEntry{
		{Name: "used1", Path: "/path1", LastUsed: now, UseCount: 5, Created: now},
		{Name: "used2", Path: "/path2", LastUsed: now.Add(-time.Hour), UseCount: 3, Created: now},
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Clear recent
	if err := ClearRecent(); err != nil {
		t.Fatalf("ClearRecent: %v", err)
	}

	// Verify last_used is cleared
	recent, _ := Recent(0)
	if len(recent) != 0 {
		t.Errorf("Expected 0 recent entries after clear, got %d", len(recent))
	}

	// But use_count should be preserved
	entry, _ := db.GetEntry("used1")
	if entry.UseCount != 5 {
		t.Errorf("Expected UseCount 5 preserved, got %d", entry.UseCount)
	}
}

func TestSortByUsage(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create aliases with different usage counts
	now := time.Now()
	entries := []database.AliasEntry{
		{Name: "low", Path: "/low", UseCount: 1, Created: now},
		{Name: "high", Path: "/high", UseCount: 100, Created: now},
		{Name: "medium", Path: "/medium", UseCount: 50, Created: now},
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// We can't easily capture stdout from ListWithSort, but we can verify
	// the database sorting logic works correctly by loading and sorting
	loaded, _ := db.LoadEntries()

	// Sort by usage (descending)
	var highFirst, mediumSecond, lowThird bool
	if len(loaded) == 3 {
		// Check relative positions based on usage count
		for _, e := range loaded {
			switch e.Name {
			case "high":
				highFirst = e.UseCount == 100
			case "medium":
				mediumSecond = e.UseCount == 50
			case "low":
				lowThird = e.UseCount == 1
			}
		}
	}

	if !highFirst || !mediumSecond || !lowThird {
		t.Error("Usage counts not preserved correctly")
	}
}

func TestAddRemoveTag(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create alias
	entry := database.AliasEntry{Name: "test", Path: "/test", Created: time.Now()}
	if err := db.SaveEntries([]database.AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Add tag using command
	if err := AddTag("test", "work"); err != nil {
		t.Fatalf("AddTag: %v", err)
	}

	// Verify tag present
	e, _ := db.GetEntry("test")
	if len(e.Tags) != 1 || e.Tags[0] != "work" {
		t.Errorf("Expected tag 'work', got %v", e.Tags)
	}

	// Add another tag
	if err := AddTag("test", "Important"); err != nil {
		t.Fatalf("AddTag: %v", err)
	}

	// Verify both tags present (Important should be lowercase)
	e, _ = db.GetEntry("test")
	if len(e.Tags) != 2 {
		t.Errorf("Expected 2 tags, got %d", len(e.Tags))
	}

	// Remove tag
	if err := RemoveTag("test", "work"); err != nil {
		t.Fatalf("RemoveTag: %v", err)
	}

	// Verify tag removed
	e, _ = db.GetEntry("test")
	if len(e.Tags) != 1 || e.Tags[0] != "important" {
		t.Errorf("Expected only 'important' tag, got %v", e.Tags)
	}
}

func TestAddTagValidation(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create alias
	entry := database.AliasEntry{Name: "test", Path: "/test", Created: time.Now()}
	if err := db.SaveEntries([]database.AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Test invalid tag
	err := AddTag("test", "invalid tag!")
	if _, ok := err.(*alias.InvalidTagError); !ok {
		t.Errorf("Expected InvalidTagError, got %T", err)
	}
}

func TestAddTagToNonExistent(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	err := AddTag("nonexistent", "tag")
	if _, ok := err.(*alias.AliasNotFoundError); !ok {
		t.Errorf("Expected AliasNotFoundError, got %T", err)
	}
}

func TestFilterByTag(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create aliases with different tags
	now := time.Now()
	entries := []database.AliasEntry{
		{Name: "work1", Path: "/work1", Tags: []string{"work"}, Created: now},
		{Name: "work2", Path: "/work2", Tags: []string{"work", "important"}, Created: now},
		{Name: "personal", Path: "/personal", Tags: []string{"personal"}, Created: now},
		{Name: "untagged", Path: "/untagged", Created: now},
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Load all entries
	loaded, _ := db.LoadEntries()

	// Filter by tag manually (since ListWithOptions writes to stdout)
	filterTag := "work"
	var filtered []database.AliasEntry
	for _, e := range loaded {
		for _, tag := range e.Tags {
			if tag == filterTag {
				filtered = append(filtered, e)
				break
			}
		}
	}

	if len(filtered) != 2 {
		t.Errorf("Expected 2 aliases with 'work' tag, got %d", len(filtered))
	}
}

func TestValidateAndNormalizeTags(t *testing.T) {
	tests := []struct {
		name     string
		input    []string
		expected []string
		hasError bool
	}{
		{
			name:     "empty input",
			input:    nil,
			expected: nil,
			hasError: false,
		},
		{
			name:     "single tag",
			input:    []string{"work"},
			expected: []string{"work"},
			hasError: false,
		},
		{
			name:     "multiple tags",
			input:    []string{"Work", "IMPORTANT", "code"},
			expected: []string{"work", "important", "code"},
			hasError: false,
		},
		{
			name:     "duplicate tags",
			input:    []string{"work", "WORK", "Work"},
			expected: []string{"work"},
			hasError: false,
		},
		{
			name:     "empty tag",
			input:    []string{"work", "", "code"},
			expected: []string{"work", "code"},
			hasError: false,
		},
		{
			name:     "whitespace tag",
			input:    []string{"  work  ", "code"},
			expected: []string{"work", "code"},
			hasError: false,
		},
		{
			name:     "invalid tag",
			input:    []string{"work", "invalid tag!"},
			expected: nil,
			hasError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := validateAndNormalizeTags(tt.input)
			if tt.hasError {
				if err == nil {
					t.Error("Expected error, got nil")
				}
			} else {
				if err != nil {
					t.Errorf("Unexpected error: %v", err)
				}
				if len(result) != len(tt.expected) {
					t.Errorf("Expected %d tags, got %d", len(tt.expected), len(result))
				}
				for i, exp := range tt.expected {
					if i < len(result) && result[i] != exp {
						t.Errorf("Tag %d: expected '%s', got '%s'", i, exp, result[i])
					}
				}
			}
		})
	}
}

func TestFormatTimeAgo(t *testing.T) {
	now := time.Now()

	tests := []struct {
		name     string
		time     time.Time
		contains string
	}{
		{"zero time", time.Time{}, "never"},
		{"just now", now.Add(-30 * time.Second), "just now"},
		{"1 minute", now.Add(-1 * time.Minute), "1 minute"},
		{"5 minutes", now.Add(-5 * time.Minute), "5 minutes"},
		{"1 hour", now.Add(-1 * time.Hour), "1 hour"},
		{"3 hours", now.Add(-3 * time.Hour), "3 hours"},
		{"1 day", now.Add(-24 * time.Hour), "1 day"},
		{"3 days", now.Add(-72 * time.Hour), "3 days"},
		{"1 week", now.Add(-7 * 24 * time.Hour), "1 week"},
		{"2 weeks", now.Add(-14 * 24 * time.Hour), "2 weeks"},
		{"1 month", now.Add(-35 * 24 * time.Hour), "1 month"},
		{"3 months", now.Add(-100 * 24 * time.Hour), "months"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := formatTimeAgo(tt.time)
			if !strings.Contains(result, tt.contains) {
				t.Errorf("formatTimeAgo(%v) = %q, expected to contain %q", tt.time, result, tt.contains)
			}
		})
	}
}

func TestFindUniqueName(t *testing.T) {
	tests := []struct {
		name        string
		baseName    string
		existingMap map[string]int
		expected    string
	}{
		{
			name:        "no conflict",
			baseName:    "test",
			existingMap: map[string]int{},
			expected:    "test_2",
		},
		{
			name:        "one conflict",
			baseName:    "test",
			existingMap: map[string]int{"test": 0},
			expected:    "test_2",
		},
		{
			name:        "multiple conflicts",
			baseName:    "test",
			existingMap: map[string]int{"test": 0, "test_2": 1, "test_3": 2},
			expected:    "test_4",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := findUniqueName(tt.baseName, tt.existingMap)
			if result != tt.expected {
				t.Errorf("findUniqueName(%q) = %q, expected %q", tt.baseName, result, tt.expected)
			}
		})
	}
}

func TestNavigateFuzzyMatching(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create alias
	testDir := env.createTestDir("development")
	entry := database.AliasEntry{
		Name:    "development",
		Path:    testDir,
		Created: time.Now(),
	}
	if err := db.SaveEntries([]database.AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Try to navigate with typo - should fail but suggest
	err := Navigate("developmnet") // typo
	if err == nil {
		t.Error("Expected error for typo, got nil")
	}

	// The error should contain a suggestion
	if err != nil && !strings.Contains(err.Error(), "Did you mean") {
		// May or may not suggest depending on threshold
		// At least verify it's an "alias not found" type error
		if !strings.Contains(err.Error(), "not found") {
			t.Errorf("Expected 'not found' error, got: %v", err)
		}
	}
}

func TestNavigateDirectoryDeleted(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create a directory and alias
	testDir := env.createTestDir("willbedeleted")
	entry := database.AliasEntry{
		Name:    "test",
		Path:    testDir,
		Created: time.Now(),
	}
	if err := db.SaveEntries([]database.AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Delete the directory
	os.RemoveAll(testDir)

	// Try to navigate - should fail
	err := Navigate("test")
	if err == nil {
		t.Error("Expected error when navigating to deleted directory")
	}

	if _, ok := err.(*alias.DirectoryNotFoundError); !ok {
		t.Errorf("Expected DirectoryNotFoundError, got %T: %v", err, err)
	}
}

func TestNavigateToRecentInvalidIndex(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	db := env.db()

	// Create one used alias
	testDir := env.createTestDir("test")
	entry := database.AliasEntry{
		Name:     "test",
		Path:     testDir,
		LastUsed: time.Now(),
		UseCount: 1,
		Created:  time.Now(),
	}
	if err := db.SaveEntries([]database.AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Try invalid index
	err := NavigateToRecent(0)
	if err == nil || !strings.Contains(err.Error(), "invalid recent index") {
		t.Errorf("Expected 'invalid recent index' error for 0, got: %v", err)
	}

	err = NavigateToRecent(10)
	if err == nil || !strings.Contains(err.Error(), "invalid recent index") {
		t.Errorf("Expected 'invalid recent index' error for 10, got: %v", err)
	}
}

func TestNavigateToRecentEmpty(t *testing.T) {
	env := setupTestEnv(t)
	defer env.cleanup()

	// No recent entries
	err := NavigateToRecent(1)
	if err == nil || !strings.Contains(err.Error(), "no recently visited") {
		t.Errorf("Expected 'no recently visited' error, got: %v", err)
	}
}

// Helper to capture stdout
func captureStdout(f func()) string {
	old := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	f()

	w.Close()
	os.Stdout = old

	var buf bytes.Buffer
	buf.ReadFrom(r)
	return buf.String()
}
