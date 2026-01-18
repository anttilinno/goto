package database

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/BurntSushi/toml"
	"github.com/antti/goto-go/internal/alias"
	"github.com/antti/goto-go/internal/fuzzy"
)

// AliasEntry represents an alias with metadata
type AliasEntry struct {
	Name     string    `toml:"name"`
	Path     string    `toml:"path"`
	Tags     []string  `toml:"tags,omitempty"`
	Created  time.Time `toml:"created"`
	LastUsed time.Time `toml:"last_used,omitempty"`
	UseCount int       `toml:"use_count"`
}

// AliasDatabase represents the TOML file structure
type AliasDatabase struct {
	Aliases []AliasEntry `toml:"aliases"`
}

// Database handles alias persistence
type Database struct {
	tomlPath string // Path to TOML file (aliases.toml)
	textPath string // Path to old text file (aliases) for migration
}

// New creates a new Database instance
// path should be the base path (e.g., ~/.config/goto/aliases)
// The TOML file will be at path + ".toml"
func New(path string) *Database {
	return &Database{
		tomlPath: path + ".toml",
		textPath: path,
	}
}

// Load reads all aliases from the database file
func (db *Database) Load() ([]alias.Alias, error) {
	entries, err := db.LoadEntries()
	if err != nil {
		return nil, err
	}

	aliases := make([]alias.Alias, len(entries))
	for i, entry := range entries {
		aliases[i] = alias.Alias{
			Name: entry.Name,
			Path: entry.Path,
		}
	}
	return aliases, nil
}

// LoadEntries reads all alias entries with metadata from the database
func (db *Database) LoadEntries() ([]AliasEntry, error) {
	// Check if TOML file exists
	if _, err := os.Stat(db.tomlPath); err == nil {
		return db.loadTOML()
	}

	// Check if old text file exists and migrate
	if _, err := os.Stat(db.textPath); err == nil {
		if err := db.migrateFromTextFormat(); err != nil {
			return nil, fmt.Errorf("migration failed: %w", err)
		}
		return db.loadTOML()
	}

	// No database exists, return empty
	return []AliasEntry{}, nil
}

// loadTOML reads aliases from the TOML file
func (db *Database) loadTOML() ([]AliasEntry, error) {
	var data AliasDatabase
	if _, err := toml.DecodeFile(db.tomlPath, &data); err != nil {
		return nil, err
	}
	return data.Aliases, nil
}

// Save writes all aliases to the database file
func (db *Database) Save(aliases []alias.Alias) error {
	// Load existing entries to preserve metadata
	existingEntries, _ := db.LoadEntries()
	existingMap := make(map[string]AliasEntry)
	for _, entry := range existingEntries {
		existingMap[entry.Name] = entry
	}

	now := time.Now()
	entries := make([]AliasEntry, len(aliases))
	for i, a := range aliases {
		if existing, ok := existingMap[a.Name]; ok {
			// Preserve existing metadata
			entries[i] = existing
			entries[i].Path = a.Path // Update path in case it changed
		} else {
			// New alias
			entries[i] = AliasEntry{
				Name:     a.Name,
				Path:     a.Path,
				Created:  now,
				UseCount: 0,
			}
		}
	}

	return db.SaveEntries(entries)
}

// SaveEntries writes all alias entries to the TOML file
func (db *Database) SaveEntries(entries []AliasEntry) error {
	// Ensure directory exists
	dir := filepath.Dir(db.tomlPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}

	file, err := os.Create(db.tomlPath)
	if err != nil {
		return err
	}
	defer file.Close()

	data := AliasDatabase{Aliases: entries}
	encoder := toml.NewEncoder(file)
	return encoder.Encode(data)
}

// Get retrieves a single alias by name
func (db *Database) Get(name string) (*alias.Alias, error) {
	aliases, err := db.Load()
	if err != nil {
		return nil, err
	}

	for _, a := range aliases {
		if a.Name == name {
			return &a, nil
		}
	}

	return nil, &alias.AliasNotFoundError{Alias: name}
}

// GetEntry retrieves a single alias entry with metadata
func (db *Database) GetEntry(name string) (*AliasEntry, error) {
	entries, err := db.LoadEntries()
	if err != nil {
		return nil, err
	}

	for _, entry := range entries {
		if entry.Name == name {
			return &entry, nil
		}
	}

	return nil, &alias.AliasNotFoundError{Alias: name}
}

// Add adds a new alias (fails if exists)
func (db *Database) Add(a alias.Alias) error {
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	// Check for duplicate
	for _, existing := range entries {
		if existing.Name == a.Name {
			return &alias.AliasExistsError{Alias: a.Name}
		}
	}

	// Add new entry with metadata
	entry := AliasEntry{
		Name:     a.Name,
		Path:     a.Path,
		Created:  time.Now(),
		UseCount: 0,
	}
	entries = append(entries, entry)
	return db.SaveEntries(entries)
}

// Remove removes an alias by name
func (db *Database) Remove(name string) error {
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	found := false
	filtered := make([]AliasEntry, 0, len(entries))
	for _, entry := range entries {
		if entry.Name == name {
			found = true
		} else {
			filtered = append(filtered, entry)
		}
	}

	if !found {
		return &alias.AliasNotFoundError{Alias: name}
	}

	return db.SaveEntries(filtered)
}

// ListNames returns just the alias names (for shell completion)
func (db *Database) ListNames() ([]string, error) {
	entries, err := db.LoadEntries()
	if err != nil {
		return nil, err
	}

	names := make([]string, len(entries))
	for i, entry := range entries {
		names[i] = entry.Name
	}
	return names, nil
}

// FindSimilar returns alias names similar to the query with similarity >= threshold.
// Results are sorted by similarity (highest first).
func (db *Database) FindSimilar(query string, threshold float64) ([]string, error) {
	names, err := db.ListNames()
	if err != nil {
		return nil, err
	}

	return fuzzy.FindSimilarNames(query, names, threshold), nil
}

// RecordUsage updates the last_used timestamp and increments use_count
func (db *Database) RecordUsage(name string) error {
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	for i, entry := range entries {
		if entry.Name == name {
			entries[i].LastUsed = time.Now()
			entries[i].UseCount++
			return db.SaveEntries(entries)
		}
	}

	return &alias.AliasNotFoundError{Alias: name}
}

// RenameAlias renames an alias while preserving all metadata
func (db *Database) RenameAlias(oldName, newName string) error {
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	// Find the alias to rename
	foundIdx := -1
	for i, entry := range entries {
		if entry.Name == oldName {
			foundIdx = i
		} else if entry.Name == newName {
			return &alias.AliasExistsError{Alias: newName}
		}
	}

	if foundIdx == -1 {
		return &alias.AliasNotFoundError{Alias: oldName}
	}

	// Rename while preserving all metadata
	entries[foundIdx].Name = newName

	return db.SaveEntries(entries)
}

// AddTag adds a tag to an alias (case-insensitive, stored lowercase)
func (db *Database) AddTag(aliasName, tag string) error {
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	for i, entry := range entries {
		if entry.Name == aliasName {
			// Check if tag already exists
			for _, existingTag := range entry.Tags {
				if existingTag == tag {
					return nil // Tag already exists, nothing to do
				}
			}
			// Add the new tag
			entries[i].Tags = append(entries[i].Tags, tag)
			return db.SaveEntries(entries)
		}
	}

	return &alias.AliasNotFoundError{Alias: aliasName}
}

// RemoveTag removes a tag from an alias
func (db *Database) RemoveTag(aliasName, tag string) error {
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	for i, entry := range entries {
		if entry.Name == aliasName {
			// Filter out the tag
			newTags := make([]string, 0, len(entry.Tags))
			found := false
			for _, existingTag := range entry.Tags {
				if existingTag == tag {
					found = true
				} else {
					newTags = append(newTags, existingTag)
				}
			}
			if !found {
				return nil // Tag not present, nothing to do
			}
			entries[i].Tags = newTags
			return db.SaveEntries(entries)
		}
	}

	return &alias.AliasNotFoundError{Alias: aliasName}
}

// SetTags replaces all tags on an alias
func (db *Database) SetTags(aliasName string, tags []string) error {
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	for i, entry := range entries {
		if entry.Name == aliasName {
			entries[i].Tags = tags
			return db.SaveEntries(entries)
		}
	}

	return &alias.AliasNotFoundError{Alias: aliasName}
}

// AddWithTags adds a new alias with tags (fails if exists)
func (db *Database) AddWithTags(a alias.Alias, tags []string) error {
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	// Check for duplicate
	for _, existing := range entries {
		if existing.Name == a.Name {
			return &alias.AliasExistsError{Alias: a.Name}
		}
	}

	// Add new entry with metadata and tags
	entry := AliasEntry{
		Name:     a.Name,
		Path:     a.Path,
		Tags:     tags,
		Created:  time.Now(),
		UseCount: 0,
	}
	entries = append(entries, entry)
	return db.SaveEntries(entries)
}

// GetAllTags returns all unique tags with their counts
func (db *Database) GetAllTags() (map[string]int, error) {
	entries, err := db.LoadEntries()
	if err != nil {
		return nil, err
	}

	tagCounts := make(map[string]int)
	for _, entry := range entries {
		for _, tag := range entry.Tags {
			tagCounts[tag]++
		}
	}
	return tagCounts, nil
}

// ClearRecentHistory resets last_used timestamps for all aliases
func (db *Database) ClearRecentHistory() error {
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	for i := range entries {
		entries[i].LastUsed = time.Time{} // Zero value
	}

	return db.SaveEntries(entries)
}

// migrateFromTextFormat migrates the old text format to TOML
func (db *Database) migrateFromTextFormat() error {
	// Read old text format
	file, err := os.Open(db.textPath)
	if err != nil {
		return err
	}
	defer file.Close()

	now := time.Now()
	var entries []AliasEntry
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())

		// Skip empty lines and comments
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		// Split on first space only (path may contain spaces)
		parts := strings.SplitN(line, " ", 2)
		if len(parts) != 2 {
			continue
		}

		entries = append(entries, AliasEntry{
			Name:     parts[0],
			Path:     parts[1],
			Created:  now,
			UseCount: 0,
		})
	}

	if err := scanner.Err(); err != nil {
		return err
	}

	// Save as TOML
	if err := db.SaveEntries(entries); err != nil {
		return err
	}

	// Backup old file
	backupPath := db.textPath + ".txt.bak"
	if err := os.Rename(db.textPath, backupPath); err != nil {
		// If rename fails, just log it but don't fail the migration
		// The TOML file is already saved
		return nil
	}

	return nil
}
