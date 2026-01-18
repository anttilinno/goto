package database

import (
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/antti/goto-go/internal/alias"
)

func TestDatabase(t *testing.T) {
	// Create temp directory
	tmpDir, err := os.MkdirTemp("", "goto-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	dbPath := filepath.Join(tmpDir, "goto")
	db := New(dbPath)

	// Test empty database
	aliases, err := db.Load()
	if err != nil {
		t.Fatalf("Load empty: %v", err)
	}
	if len(aliases) != 0 {
		t.Errorf("Expected 0 aliases, got %d", len(aliases))
	}

	// Test Add
	testAlias := alias.Alias{Name: "test", Path: "/tmp"}
	if err := db.Add(testAlias); err != nil {
		t.Fatalf("Add: %v", err)
	}

	// Test Get
	got, err := db.Get("test")
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	if got.Path != "/tmp" {
		t.Errorf("Expected /tmp, got %s", got.Path)
	}

	// Test duplicate detection
	err = db.Add(testAlias)
	if _, ok := err.(*alias.AliasExistsError); !ok {
		t.Errorf("Expected AliasExistsError, got %T", err)
	}

	// Test Remove
	if err := db.Remove("test"); err != nil {
		t.Fatalf("Remove: %v", err)
	}

	// Test not found
	_, err = db.Get("test")
	if _, ok := err.(*alias.AliasNotFoundError); !ok {
		t.Errorf("Expected AliasNotFoundError, got %T", err)
	}
}

func TestDatabasePathsWithSpaces(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "goto-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	dbPath := filepath.Join(tmpDir, "goto")
	db := New(dbPath)

	// Create directory with spaces
	spacePath := filepath.Join(tmpDir, "path with spaces")
	if err := os.Mkdir(spacePath, 0755); err != nil {
		t.Fatal(err)
	}

	testAlias := alias.Alias{Name: "spaces", Path: spacePath}
	if err := db.Add(testAlias); err != nil {
		t.Fatalf("Add: %v", err)
	}

	got, err := db.Get("spaces")
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	if got.Path != spacePath {
		t.Errorf("Expected %s, got %s", spacePath, got.Path)
	}
}

func TestDatabaseMetadata(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "goto-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	dbPath := filepath.Join(tmpDir, "goto")
	db := New(dbPath)

	// Add an alias
	testAlias := alias.Alias{Name: "test", Path: "/tmp"}
	if err := db.Add(testAlias); err != nil {
		t.Fatalf("Add: %v", err)
	}

	// Check that metadata was set
	entry, err := db.GetEntry("test")
	if err != nil {
		t.Fatalf("GetEntry: %v", err)
	}

	if entry.UseCount != 0 {
		t.Errorf("Expected UseCount 0, got %d", entry.UseCount)
	}

	if entry.Created.IsZero() {
		t.Error("Expected Created to be set")
	}

	// Record usage
	if err := db.RecordUsage("test"); err != nil {
		t.Fatalf("RecordUsage: %v", err)
	}

	// Check updated metadata
	entry, err = db.GetEntry("test")
	if err != nil {
		t.Fatalf("GetEntry after usage: %v", err)
	}

	if entry.UseCount != 1 {
		t.Errorf("Expected UseCount 1, got %d", entry.UseCount)
	}

	if entry.LastUsed.IsZero() {
		t.Error("Expected LastUsed to be set")
	}
}

func TestDatabaseMigration(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "goto-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	// Create old-style text database
	textPath := filepath.Join(tmpDir, "goto")
	textContent := "dev /home/user/dev\nblog /var/www/html/blog\nspaces /path/with spaces\n"
	if err := os.WriteFile(textPath, []byte(textContent), 0644); err != nil {
		t.Fatal(err)
	}

	// Create database instance (will trigger migration)
	db := New(textPath)
	aliases, err := db.Load()
	if err != nil {
		t.Fatalf("Load (migration): %v", err)
	}

	if len(aliases) != 3 {
		t.Errorf("Expected 3 aliases, got %d", len(aliases))
	}

	// Verify aliases
	expected := map[string]string{
		"dev":    "/home/user/dev",
		"blog":   "/var/www/html/blog",
		"spaces": "/path/with spaces",
	}

	for _, a := range aliases {
		expectedPath, ok := expected[a.Name]
		if !ok {
			t.Errorf("Unexpected alias: %s", a.Name)
			continue
		}
		if a.Path != expectedPath {
			t.Errorf("Alias %s: expected path %s, got %s", a.Name, expectedPath, a.Path)
		}
	}

	// Verify TOML file was created
	tomlPath := textPath + ".toml"
	if _, err := os.Stat(tomlPath); os.IsNotExist(err) {
		t.Error("TOML file was not created")
	}

	// Verify backup was created
	backupPath := textPath + ".txt.bak"
	if _, err := os.Stat(backupPath); os.IsNotExist(err) {
		t.Error("Backup file was not created")
	}

	// Verify original text file no longer exists
	if _, err := os.Stat(textPath); !os.IsNotExist(err) {
		t.Error("Original text file should have been renamed")
	}

	// Verify entries have metadata
	entries, err := db.LoadEntries()
	if err != nil {
		t.Fatalf("LoadEntries: %v", err)
	}

	for _, entry := range entries {
		if entry.Created.IsZero() {
			t.Errorf("Entry %s: Created should be set", entry.Name)
		}
		if entry.UseCount != 0 {
			t.Errorf("Entry %s: expected UseCount 0, got %d", entry.Name, entry.UseCount)
		}
	}
}

func TestDatabaseTOMLFormat(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "goto-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	dbPath := filepath.Join(tmpDir, "goto")
	db := New(dbPath)

	// Add alias with tags
	now := time.Now()
	entry := AliasEntry{
		Name:     "work",
		Path:     "/home/user/work",
		Tags:     []string{"work", "important"},
		Created:  now,
		LastUsed: now,
		UseCount: 42,
	}

	if err := db.SaveEntries([]AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Reload and verify
	entries, err := db.LoadEntries()
	if err != nil {
		t.Fatalf("LoadEntries: %v", err)
	}

	if len(entries) != 1 {
		t.Fatalf("Expected 1 entry, got %d", len(entries))
	}

	loaded := entries[0]
	if loaded.Name != "work" {
		t.Errorf("Expected name 'work', got '%s'", loaded.Name)
	}
	if loaded.Path != "/home/user/work" {
		t.Errorf("Expected path '/home/user/work', got '%s'", loaded.Path)
	}
	if len(loaded.Tags) != 2 || loaded.Tags[0] != "work" || loaded.Tags[1] != "important" {
		t.Errorf("Tags mismatch: %v", loaded.Tags)
	}
	if loaded.UseCount != 42 {
		t.Errorf("Expected UseCount 42, got %d", loaded.UseCount)
	}
}

func TestDatabaseRecordUsageNotFound(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "goto-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	dbPath := filepath.Join(tmpDir, "goto")
	db := New(dbPath)

	// Try to record usage for non-existent alias
	err = db.RecordUsage("nonexistent")
	if _, ok := err.(*alias.AliasNotFoundError); !ok {
		t.Errorf("Expected AliasNotFoundError, got %T", err)
	}
}

func TestMigrateFromTextFormatWithComments(t *testing.T) {
	tmpDir := t.TempDir()

	// Create old-style text database with comments and empty lines
	textPath := filepath.Join(tmpDir, "aliases")
	textContent := `# This is a comment
dev /home/user/dev

# Another comment
blog /var/www/html/blog

projects /home/user/projects
`
	if err := os.WriteFile(textPath, []byte(textContent), 0644); err != nil {
		t.Fatal(err)
	}

	db := New(textPath)
	aliases, err := db.Load()
	if err != nil {
		t.Fatalf("Load (migration): %v", err)
	}

	if len(aliases) != 3 {
		t.Errorf("Expected 3 aliases, got %d", len(aliases))
	}

	// Verify backup was created
	backupPath := textPath + ".txt.bak"
	if _, err := os.Stat(backupPath); os.IsNotExist(err) {
		t.Error("Backup file was not created")
	}
}

func TestLoadExistingTOML(t *testing.T) {
	tmpDir := t.TempDir()

	// Create TOML file with metadata directly
	tomlPath := filepath.Join(tmpDir, "aliases.toml")
	tomlContent := `[[aliases]]
name = "dev"
path = "/home/user/dev"
tags = ["work", "code"]
created = 2024-01-01T10:00:00Z
last_used = 2024-06-15T14:30:00Z
use_count = 42

[[aliases]]
name = "docs"
path = "/home/user/docs"
created = 2024-02-15T12:00:00Z
use_count = 10
`
	if err := os.WriteFile(tomlPath, []byte(tomlContent), 0644); err != nil {
		t.Fatal(err)
	}

	// Use the base path without .toml extension
	basePath := filepath.Join(tmpDir, "aliases")
	db := New(basePath)

	entries, err := db.LoadEntries()
	if err != nil {
		t.Fatalf("LoadEntries: %v", err)
	}

	if len(entries) != 2 {
		t.Fatalf("Expected 2 entries, got %d", len(entries))
	}

	// Verify first entry
	dev := entries[0]
	if dev.Name != "dev" {
		t.Errorf("Expected name 'dev', got '%s'", dev.Name)
	}
	if dev.UseCount != 42 {
		t.Errorf("Expected UseCount 42, got %d", dev.UseCount)
	}
	if len(dev.Tags) != 2 || dev.Tags[0] != "work" || dev.Tags[1] != "code" {
		t.Errorf("Tags mismatch: %v", dev.Tags)
	}
	if dev.LastUsed.IsZero() {
		t.Error("Expected LastUsed to be set")
	}
}

func TestRenameAlias(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Create alias with metadata
	now := time.Now()
	entry := AliasEntry{
		Name:     "old-name",
		Path:     "/home/user/project",
		Tags:     []string{"work", "important"},
		Created:  now.Add(-24 * time.Hour),
		LastUsed: now,
		UseCount: 15,
	}
	if err := db.SaveEntries([]AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Rename alias
	if err := db.RenameAlias("old-name", "new-name"); err != nil {
		t.Fatalf("RenameAlias: %v", err)
	}

	// Verify old name is gone
	_, err := db.Get("old-name")
	if _, ok := err.(*alias.AliasNotFoundError); !ok {
		t.Errorf("Expected AliasNotFoundError for old name, got %T", err)
	}

	// Verify new name exists with metadata preserved
	renamed, err := db.GetEntry("new-name")
	if err != nil {
		t.Fatalf("GetEntry for new name: %v", err)
	}

	if renamed.Path != "/home/user/project" {
		t.Errorf("Expected path '/home/user/project', got '%s'", renamed.Path)
	}
	if renamed.UseCount != 15 {
		t.Errorf("Expected UseCount 15, got %d", renamed.UseCount)
	}
	if len(renamed.Tags) != 2 {
		t.Errorf("Expected 2 tags, got %d", len(renamed.Tags))
	}
}

func TestRenameErrors(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Create two aliases
	entries := []AliasEntry{
		{Name: "first", Path: "/first", Created: time.Now()},
		{Name: "second", Path: "/second", Created: time.Now()},
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Test rename non-existent
	err := db.RenameAlias("nonexistent", "newname")
	if _, ok := err.(*alias.AliasNotFoundError); !ok {
		t.Errorf("Expected AliasNotFoundError, got %T", err)
	}

	// Test rename to existing name
	err = db.RenameAlias("first", "second")
	if _, ok := err.(*alias.AliasExistsError); !ok {
		t.Errorf("Expected AliasExistsError, got %T", err)
	}
}

func TestAddRemoveTag(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Create alias without tags
	entry := AliasEntry{Name: "test", Path: "/test", Created: time.Now()}
	if err := db.SaveEntries([]AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Add tag
	if err := db.AddTag("test", "work"); err != nil {
		t.Fatalf("AddTag: %v", err)
	}

	// Verify tag present
	e, _ := db.GetEntry("test")
	if len(e.Tags) != 1 || e.Tags[0] != "work" {
		t.Errorf("Expected tag 'work', got %v", e.Tags)
	}

	// Add duplicate tag (should be no-op)
	if err := db.AddTag("test", "work"); err != nil {
		t.Fatalf("AddTag duplicate: %v", err)
	}
	e, _ = db.GetEntry("test")
	if len(e.Tags) != 1 {
		t.Errorf("Expected 1 tag after duplicate add, got %d", len(e.Tags))
	}

	// Add another tag
	if err := db.AddTag("test", "important"); err != nil {
		t.Fatalf("AddTag second: %v", err)
	}
	e, _ = db.GetEntry("test")
	if len(e.Tags) != 2 {
		t.Errorf("Expected 2 tags, got %d", len(e.Tags))
	}

	// Remove tag
	if err := db.RemoveTag("test", "work"); err != nil {
		t.Fatalf("RemoveTag: %v", err)
	}

	// Verify tag removed
	e, _ = db.GetEntry("test")
	if len(e.Tags) != 1 || e.Tags[0] != "important" {
		t.Errorf("Expected only 'important' tag, got %v", e.Tags)
	}

	// Remove non-existent tag (should be no-op)
	if err := db.RemoveTag("test", "nonexistent"); err != nil {
		t.Fatalf("RemoveTag nonexistent: %v", err)
	}
}

func TestAddTagToNonExistent(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	err := db.AddTag("nonexistent", "tag")
	if _, ok := err.(*alias.AliasNotFoundError); !ok {
		t.Errorf("Expected AliasNotFoundError, got %T", err)
	}
}

func TestSetTags(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Create alias with tags
	entry := AliasEntry{
		Name:    "test",
		Path:    "/test",
		Tags:    []string{"old1", "old2"},
		Created: time.Now(),
	}
	if err := db.SaveEntries([]AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Replace all tags
	if err := db.SetTags("test", []string{"new1", "new2", "new3"}); err != nil {
		t.Fatalf("SetTags: %v", err)
	}

	e, _ := db.GetEntry("test")
	if len(e.Tags) != 3 || e.Tags[0] != "new1" {
		t.Errorf("Expected new tags, got %v", e.Tags)
	}

	// Clear all tags
	if err := db.SetTags("test", []string{}); err != nil {
		t.Fatalf("SetTags empty: %v", err)
	}

	e, _ = db.GetEntry("test")
	if len(e.Tags) != 0 {
		t.Errorf("Expected no tags, got %v", e.Tags)
	}
}

func TestGetAllTags(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Create aliases with various tags
	entries := []AliasEntry{
		{Name: "a1", Path: "/a1", Tags: []string{"work", "code"}, Created: time.Now()},
		{Name: "a2", Path: "/a2", Tags: []string{"work", "docs"}, Created: time.Now()},
		{Name: "a3", Path: "/a3", Tags: []string{"personal"}, Created: time.Now()},
		{Name: "a4", Path: "/a4", Created: time.Now()}, // no tags
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	tagCounts, err := db.GetAllTags()
	if err != nil {
		t.Fatalf("GetAllTags: %v", err)
	}

	if tagCounts["work"] != 2 {
		t.Errorf("Expected work=2, got %d", tagCounts["work"])
	}
	if tagCounts["code"] != 1 {
		t.Errorf("Expected code=1, got %d", tagCounts["code"])
	}
	if tagCounts["docs"] != 1 {
		t.Errorf("Expected docs=1, got %d", tagCounts["docs"])
	}
	if tagCounts["personal"] != 1 {
		t.Errorf("Expected personal=1, got %d", tagCounts["personal"])
	}
	if len(tagCounts) != 4 {
		t.Errorf("Expected 4 unique tags, got %d", len(tagCounts))
	}
}

func TestAddWithTags(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Add alias with tags
	a := alias.Alias{Name: "test", Path: "/test"}
	if err := db.AddWithTags(a, []string{"work", "important"}); err != nil {
		t.Fatalf("AddWithTags: %v", err)
	}

	entry, err := db.GetEntry("test")
	if err != nil {
		t.Fatalf("GetEntry: %v", err)
	}

	if len(entry.Tags) != 2 {
		t.Errorf("Expected 2 tags, got %d", len(entry.Tags))
	}
	if entry.Tags[0] != "work" || entry.Tags[1] != "important" {
		t.Errorf("Expected ['work', 'important'], got %v", entry.Tags)
	}
}

func TestClearRecentHistory(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Create aliases with usage data
	now := time.Now()
	entries := []AliasEntry{
		{Name: "a1", Path: "/a1", LastUsed: now, UseCount: 5, Created: now},
		{Name: "a2", Path: "/a2", LastUsed: now.Add(-time.Hour), UseCount: 3, Created: now},
		{Name: "a3", Path: "/a3", Created: now}, // never used
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Clear recent history
	if err := db.ClearRecentHistory(); err != nil {
		t.Fatalf("ClearRecentHistory: %v", err)
	}

	// Verify last_used is cleared but use_count preserved
	updated, _ := db.LoadEntries()
	for _, e := range updated {
		if !e.LastUsed.IsZero() {
			t.Errorf("Expected LastUsed to be zero for %s, got %v", e.Name, e.LastUsed)
		}
		// UseCount should NOT be cleared
		if e.Name == "a1" && e.UseCount != 5 {
			t.Errorf("Expected UseCount 5 for a1, got %d", e.UseCount)
		}
	}
}

func TestRecordUsageIncrementsCounter(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Create alias
	entry := AliasEntry{Name: "test", Path: "/test", Created: time.Now()}
	if err := db.SaveEntries([]AliasEntry{entry}); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Record usage multiple times
	for i := 0; i < 5; i++ {
		if err := db.RecordUsage("test"); err != nil {
			t.Fatalf("RecordUsage %d: %v", i, err)
		}
	}

	// Verify count
	e, _ := db.GetEntry("test")
	if e.UseCount != 5 {
		t.Errorf("Expected UseCount 5, got %d", e.UseCount)
	}
	if e.LastUsed.IsZero() {
		t.Error("Expected LastUsed to be set")
	}
}

func TestFindSimilar(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Create aliases
	entries := []AliasEntry{
		{Name: "dev", Path: "/dev", Created: time.Now()},
		{Name: "development", Path: "/development", Created: time.Now()},
		{Name: "projects", Path: "/projects", Created: time.Now()},
		{Name: "docs", Path: "/docs", Created: time.Now()},
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	// Test fuzzy matching with typo
	suggestions, err := db.FindSimilar("dve", 0.3)
	if err != nil {
		t.Fatalf("FindSimilar: %v", err)
	}

	if len(suggestions) == 0 {
		t.Error("Expected suggestions for 'dve', got none")
	}
	if suggestions[0] != "dev" {
		t.Errorf("Expected first suggestion 'dev', got '%s'", suggestions[0])
	}

	// Test substring matching
	suggestions, err = db.FindSimilar("proj", 0.5)
	if err != nil {
		t.Fatalf("FindSimilar substring: %v", err)
	}

	found := false
	for _, s := range suggestions {
		if s == "projects" {
			found = true
			break
		}
	}
	if !found {
		t.Errorf("Expected 'projects' in suggestions for 'proj', got %v", suggestions)
	}
}

func TestDatabaseListNames(t *testing.T) {
	tmpDir := t.TempDir()

	dbPath := filepath.Join(tmpDir, "aliases")
	db := New(dbPath)

	// Test empty database
	names, err := db.ListNames()
	if err != nil {
		t.Fatalf("ListNames empty: %v", err)
	}
	if len(names) != 0 {
		t.Errorf("Expected 0 names, got %d", len(names))
	}

	// Add entries
	entries := []AliasEntry{
		{Name: "alpha", Path: "/a", Created: time.Now()},
		{Name: "beta", Path: "/b", Created: time.Now()},
		{Name: "gamma", Path: "/c", Created: time.Now()},
	}
	if err := db.SaveEntries(entries); err != nil {
		t.Fatalf("SaveEntries: %v", err)
	}

	names, err = db.ListNames()
	if err != nil {
		t.Fatalf("ListNames: %v", err)
	}
	if len(names) != 3 {
		t.Errorf("Expected 3 names, got %d", len(names))
	}
}
