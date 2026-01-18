package commands

import (
	"fmt"
	"os"
	"sort"
	"strings"
	"text/tabwriter"
	"time"

	"github.com/BurntSushi/toml"
	"github.com/antti/goto-go/internal/alias"
	"github.com/antti/goto-go/internal/config"
	"github.com/antti/goto-go/internal/database"
	"github.com/antti/goto-go/internal/stack"
)

// Register creates a new alias for a directory
func Register(aliasName, directory string) error {
	return RegisterWithTags(aliasName, directory, nil)
}

// RegisterWithTags creates a new alias for a directory with optional tags
func RegisterWithTags(aliasName, directory string, tags []string) error {
	// Validate alias name
	if err := alias.Validate(aliasName); err != nil {
		return err
	}

	// Validate and normalize tags
	normalizedTags, err := validateAndNormalizeTags(tags)
	if err != nil {
		return err
	}

	// Expand and validate directory
	expandedPath, err := config.ExpandPath(directory)
	if err != nil {
		return err
	}

	// Check directory exists
	info, err := os.Stat(expandedPath)
	if os.IsNotExist(err) {
		return &alias.DirectoryNotFoundError{Path: expandedPath}
	}
	if err != nil {
		return err
	}
	if !info.IsDir() {
		return fmt.Errorf("not a directory: %s", expandedPath)
	}

	// Load config and database
	cfg, err := config.Load()
	if err != nil {
		return err
	}
	if err := cfg.EnsureConfigDir(); err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)

	// Add alias with tags
	if err := db.AddWithTags(alias.Alias{Name: aliasName, Path: expandedPath}, normalizedTags); err != nil {
		return err
	}

	if len(normalizedTags) > 0 {
		fmt.Printf("Registered '%s' -> %s [%s]\n", aliasName, expandedPath, strings.Join(normalizedTags, ", "))
	} else {
		fmt.Printf("Registered '%s' -> %s\n", aliasName, expandedPath)
	}
	return nil
}

// validateAndNormalizeTags validates tags and converts them to lowercase
func validateAndNormalizeTags(tags []string) ([]string, error) {
	if len(tags) == 0 {
		return nil, nil
	}

	normalized := make([]string, 0, len(tags))
	seen := make(map[string]bool)

	for _, tag := range tags {
		tag = strings.ToLower(strings.TrimSpace(tag))
		if tag == "" {
			continue
		}
		if err := alias.ValidateTag(tag); err != nil {
			return nil, err
		}
		if !seen[tag] {
			seen[tag] = true
			normalized = append(normalized, tag)
		}
	}

	return normalized, nil
}

// Unregister removes an alias
func Unregister(aliasName string) error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)

	if err := db.Remove(aliasName); err != nil {
		return err
	}

	fmt.Printf("Unregistered '%s'\n", aliasName)
	return nil
}

// SortOrder represents the sort order for listing aliases
type SortOrder string

const (
	SortAlpha  SortOrder = "alpha"
	SortUsage  SortOrder = "usage"
	SortRecent SortOrder = "recent"
)

// List displays all registered aliases
func List() error {
	return ListWithOptions("", "")
}

// ListWithSort displays all registered aliases with the specified sort order
func ListWithSort(sortOrder string) error {
	return ListWithOptions(sortOrder, "")
}

// ListWithOptions displays aliases with optional sorting and filtering by tag
func ListWithOptions(sortOrder, filterTag string) error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	// Filter by tag if specified
	if filterTag != "" {
		filterTag = strings.ToLower(filterTag)
		filtered := make([]database.AliasEntry, 0)
		for _, entry := range entries {
			for _, tag := range entry.Tags {
				if tag == filterTag {
					filtered = append(filtered, entry)
					break
				}
			}
		}
		entries = filtered
	}

	if len(entries) == 0 {
		if filterTag != "" {
			fmt.Printf("No aliases with tag '%s'\n", filterTag)
		} else {
			fmt.Println("No aliases registered")
		}
		return nil
	}

	// Determine sort order (use config default if not specified)
	if sortOrder == "" {
		sortOrder = cfg.User.General.DefaultSort
	}

	// Sort entries based on sort order
	switch SortOrder(sortOrder) {
	case SortUsage:
		sort.Slice(entries, func(i, j int) bool {
			return entries[i].UseCount > entries[j].UseCount
		})
	case SortRecent:
		sort.Slice(entries, func(i, j int) bool {
			return entries[i].LastUsed.After(entries[j].LastUsed)
		})
	case SortAlpha:
		fallthrough
	default:
		sort.Slice(entries, func(i, j int) bool {
			return entries[i].Name < entries[j].Name
		})
	}

	// Use tabwriter for aligned columns
	w := tabwriter.NewWriter(os.Stdout, 0, 0, 4, ' ', 0)
	for _, entry := range entries {
		tagsStr := ""
		if cfg.User.Display.ShowTags {
			if len(entry.Tags) > 0 {
				tagsStr = fmt.Sprintf("\t[%s]", strings.Join(entry.Tags, ", "))
			} else {
				tagsStr = "\t[]"
			}
		}

		if cfg.User.Display.ShowStats {
			fmt.Fprintf(w, "%s\t%s\t[%d uses]%s\n", entry.Name, entry.Path, entry.UseCount, tagsStr)
		} else {
			fmt.Fprintf(w, "%s\t%s%s\n", entry.Name, entry.Path, tagsStr)
		}
	}
	w.Flush()

	return nil
}

// Expand prints the path for an alias (no decoration)
func Expand(aliasName string) error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	a, err := db.Get(aliasName)
	if err != nil {
		return err
	}

	fmt.Println(a.Path)
	return nil
}

// ListAliasNames prints alias names only (for shell completion)
func ListAliasNames() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	names, err := db.ListNames()
	if err != nil {
		return err
	}

	for _, name := range names {
		fmt.Println(name)
	}
	return nil
}

// Navigate looks up an alias and prints the path for shell to cd
func Navigate(aliasName string) error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	a, err := db.Get(aliasName)
	if err != nil {
		// If alias not found, try fuzzy matching
		if _, ok := err.(*alias.AliasNotFoundError); ok {
			suggestions, fuzzyErr := db.FindSimilar(aliasName, cfg.User.General.FuzzyThreshold)
			if fuzzyErr == nil && len(suggestions) > 0 {
				// Limit to top 3 suggestions
				if len(suggestions) > 3 {
					suggestions = suggestions[:3]
				}
				return fmt.Errorf("alias '%s' not found. Did you mean: %s?", aliasName, strings.Join(suggestions, ", "))
			}
		}
		return err
	}

	// Verify directory still exists
	info, err := os.Stat(a.Path)
	if os.IsNotExist(err) {
		return &alias.DirectoryNotFoundError{Path: a.Path}
	}
	if err != nil {
		return err
	}
	if !info.IsDir() {
		return fmt.Errorf("not a directory: %s", a.Path)
	}

	// Record usage (increment use_count, update last_used)
	_ = db.RecordUsage(aliasName)

	// Output path for shell wrapper to use
	fmt.Println(a.Path)
	return nil
}

// Cleanup removes aliases pointing to non-existent directories
func Cleanup() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	aliases, err := db.Load()
	if err != nil {
		return err
	}

	var valid []alias.Alias
	var removed []string

	for _, a := range aliases {
		info, err := os.Stat(a.Path)
		if err != nil || !info.IsDir() {
			removed = append(removed, a.Name)
		} else {
			valid = append(valid, a)
		}
	}

	if len(removed) == 0 {
		fmt.Println("Nothing to clean up")
		return nil
	}

	// Save filtered list
	if err := db.Save(valid); err != nil {
		return err
	}

	fmt.Println("Removed aliases:")
	for _, name := range removed {
		fmt.Printf("  - %s\n", name)
	}

	return nil
}

// Push saves current directory to stack, then navigates to alias
func Push(aliasName string) error {
	// Get current working directory
	cwd, err := os.Getwd()
	if err != nil {
		return err
	}

	cfg, err := config.Load()
	if err != nil {
		return err
	}

	// Look up alias first (fail early if not found)
	db := database.New(cfg.DatabasePath)
	a, err := db.Get(aliasName)
	if err != nil {
		return err
	}

	// Verify target directory exists
	info, err := os.Stat(a.Path)
	if os.IsNotExist(err) {
		return &alias.DirectoryNotFoundError{Path: a.Path}
	}
	if err != nil {
		return err
	}
	if !info.IsDir() {
		return fmt.Errorf("not a directory: %s", a.Path)
	}

	// Push current directory to stack
	s := stack.New(cfg.StackPath)
	if err := s.Push(cwd); err != nil {
		return err
	}

	// Record usage (increment use_count, update last_used)
	_ = db.RecordUsage(aliasName)

	// Output target path for shell
	fmt.Println(a.Path)
	return nil
}

// Pop retrieves directory from stack and outputs it for shell to cd
func Pop() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	s := stack.New(cfg.StackPath)
	dir, err := s.Pop()
	if err != nil {
		return err
	}

	// Verify directory still exists
	info, err := os.Stat(dir)
	if os.IsNotExist(err) {
		return &alias.DirectoryNotFoundError{Path: dir}
	}
	if err != nil {
		return err
	}
	if !info.IsDir() {
		return fmt.Errorf("not a directory: %s", dir)
	}

	fmt.Println(dir)
	return nil
}

// ShowConfig displays the current configuration
func ShowConfig() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	fmt.Print(cfg.FormatConfig())
	return nil
}

// Export outputs all aliases as TOML to stdout
func Export() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	if len(entries) == 0 {
		fmt.Fprintln(os.Stderr, "No aliases to export")
		return nil
	}

	data := database.AliasDatabase{Aliases: entries}
	encoder := toml.NewEncoder(os.Stdout)
	return encoder.Encode(data)
}

// ImportResult contains statistics about the import operation
type ImportResult struct {
	Imported int
	Skipped  int
	Renamed  int
	Warnings []string
}

// Import reads aliases from a TOML file and merges with existing database
func Import(filepath string, strategy string) (*ImportResult, error) {
	// Validate strategy
	if strategy != "skip" && strategy != "overwrite" && strategy != "rename" {
		return nil, fmt.Errorf("invalid strategy: %s (must be skip, overwrite, or rename)", strategy)
	}

	// Read and parse import file
	var importData database.AliasDatabase
	if _, err := toml.DecodeFile(filepath, &importData); err != nil {
		return nil, fmt.Errorf("failed to parse import file: %w", err)
	}

	if len(importData.Aliases) == 0 {
		return nil, fmt.Errorf("no aliases found in import file")
	}

	// Load existing database
	cfg, err := config.Load()
	if err != nil {
		return nil, err
	}
	if err := cfg.EnsureConfigDir(); err != nil {
		return nil, err
	}

	db := database.New(cfg.DatabasePath)
	existingEntries, err := db.LoadEntries()
	if err != nil {
		return nil, err
	}

	// Build map of existing aliases
	existingMap := make(map[string]int) // name -> index in existingEntries
	for i, entry := range existingEntries {
		existingMap[entry.Name] = i
	}

	result := &ImportResult{}

	// Process each imported alias
	for _, importEntry := range importData.Aliases {
		// Validate alias name
		if err := alias.Validate(importEntry.Name); err != nil {
			result.Warnings = append(result.Warnings, fmt.Sprintf("skipping invalid alias name '%s': %v", importEntry.Name, err))
			result.Skipped++
			continue
		}

		// Check if path exists (warn but don't skip)
		if _, err := os.Stat(importEntry.Path); os.IsNotExist(err) {
			result.Warnings = append(result.Warnings, fmt.Sprintf("warning: path does not exist for alias '%s': %s", importEntry.Name, importEntry.Path))
		}

		if idx, exists := existingMap[importEntry.Name]; exists {
			// Alias already exists - handle based on strategy
			switch strategy {
			case "skip":
				result.Skipped++
			case "overwrite":
				existingEntries[idx] = importEntry
				result.Imported++
			case "rename":
				// Find a unique name with suffix
				newName := findUniqueName(importEntry.Name, existingMap)
				importEntry.Name = newName
				existingEntries = append(existingEntries, importEntry)
				existingMap[newName] = len(existingEntries) - 1
				result.Renamed++
			}
		} else {
			// New alias - add it
			existingEntries = append(existingEntries, importEntry)
			existingMap[importEntry.Name] = len(existingEntries) - 1
			result.Imported++
		}
	}

	// Save the merged database
	if err := db.SaveEntries(existingEntries); err != nil {
		return nil, fmt.Errorf("failed to save database: %w", err)
	}

	return result, nil
}

// findUniqueName generates a unique alias name by appending a numeric suffix
func findUniqueName(baseName string, existingMap map[string]int) string {
	suffix := 2
	for {
		newName := fmt.Sprintf("%s_%d", baseName, suffix)
		if _, exists := existingMap[newName]; !exists {
			return newName
		}
		suffix++
	}
}

// Stats displays usage statistics for aliases
func Stats() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	entries, err := db.LoadEntries()
	if err != nil {
		return err
	}

	if len(entries) == 0 {
		fmt.Println("No aliases registered")
		return nil
	}

	// Sort by use count descending
	sortedEntries := make([]database.AliasEntry, len(entries))
	copy(sortedEntries, entries)
	sort.Slice(sortedEntries, func(i, j int) bool {
		return sortedEntries[i].UseCount > sortedEntries[j].UseCount
	})

	// Calculate total navigations
	totalNavigations := 0
	for _, entry := range entries {
		totalNavigations += entry.UseCount
	}

	fmt.Println("Usage Statistics")
	fmt.Println("================")
	fmt.Println("Most Used:")

	// Show top aliases (up to 10)
	limit := 10
	if len(sortedEntries) < limit {
		limit = len(sortedEntries)
	}

	for i := 0; i < limit; i++ {
		entry := sortedEntries[i]
		if entry.UseCount == 0 {
			break // Stop showing if no uses
		}
		lastUsedStr := formatTimeAgo(entry.LastUsed)
		fmt.Printf("  %d. %-12s (%d uses, last: %s)\n", i+1, entry.Name, entry.UseCount, lastUsedStr)
	}

	fmt.Println()
	fmt.Printf("Total aliases: %d\n", len(entries))
	fmt.Printf("Total navigations: %d\n", totalNavigations)

	return nil
}

// formatTimeAgo returns a human-readable time difference string
func formatTimeAgo(t time.Time) string {
	if t.IsZero() {
		return "never"
	}

	duration := time.Since(t)

	switch {
	case duration < time.Minute:
		return "just now"
	case duration < time.Hour:
		mins := int(duration.Minutes())
		if mins == 1 {
			return "1 minute ago"
		}
		return fmt.Sprintf("%d minutes ago", mins)
	case duration < 24*time.Hour:
		hours := int(duration.Hours())
		if hours == 1 {
			return "1 hour ago"
		}
		return fmt.Sprintf("%d hours ago", hours)
	case duration < 7*24*time.Hour:
		days := int(duration.Hours() / 24)
		if days == 1 {
			return "1 day ago"
		}
		return fmt.Sprintf("%d days ago", days)
	case duration < 30*24*time.Hour:
		weeks := int(duration.Hours() / 24 / 7)
		if weeks == 1 {
			return "1 week ago"
		}
		return fmt.Sprintf("%d weeks ago", weeks)
	default:
		months := int(duration.Hours() / 24 / 30)
		if months == 1 {
			return "1 month ago"
		}
		return fmt.Sprintf("%d months ago", months)
	}
}

// Rename renames an alias while preserving all metadata
func Rename(oldName, newName string) error {
	// Validate new alias name
	if err := alias.Validate(newName); err != nil {
		return err
	}

	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)

	if err := db.RenameAlias(oldName, newName); err != nil {
		return err
	}

	fmt.Printf("Renamed alias '%s' to '%s'\n", oldName, newName)
	return nil
}

// AddTag adds a tag to an alias
func AddTag(aliasName, tag string) error {
	// Validate and normalize the tag
	tag = strings.ToLower(strings.TrimSpace(tag))
	if err := alias.ValidateTag(tag); err != nil {
		return err
	}

	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)

	if err := db.AddTag(aliasName, tag); err != nil {
		return err
	}

	fmt.Printf("Added tag '%s' to alias '%s'\n", tag, aliasName)
	return nil
}

// RemoveTag removes a tag from an alias
func RemoveTag(aliasName, tag string) error {
	tag = strings.ToLower(strings.TrimSpace(tag))

	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)

	if err := db.RemoveTag(aliasName, tag); err != nil {
		return err
	}

	fmt.Printf("Removed tag '%s' from alias '%s'\n", tag, aliasName)
	return nil
}

// ListTags shows all unique tags with counts
func ListTags() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	tagCounts, err := db.GetAllTags()
	if err != nil {
		return err
	}

	if len(tagCounts) == 0 {
		fmt.Println("No tags found")
		return nil
	}

	// Sort tags alphabetically
	tags := make([]string, 0, len(tagCounts))
	for tag := range tagCounts {
		tags = append(tags, tag)
	}
	sort.Strings(tags)

	fmt.Println("Tags:")
	for _, tag := range tags {
		count := tagCounts[tag]
		if count == 1 {
			fmt.Printf("  %-12s (%d alias)\n", tag, count)
		} else {
			fmt.Printf("  %-12s (%d aliases)\n", tag, count)
		}
	}

	return nil
}

// ListTagsRaw outputs just tag names, one per line (for shell completion)
func ListTagsRaw() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	tagCounts, err := db.GetAllTags()
	if err != nil {
		return err
	}

	// Sort tags alphabetically
	tags := make([]string, 0, len(tagCounts))
	for tag := range tagCounts {
		tags = append(tags, tag)
	}
	sort.Strings(tags)

	for _, tag := range tags {
		fmt.Println(tag)
	}
	return nil
}

// RecentEntry represents a recently visited alias
type RecentEntry struct {
	Alias    string
	Path     string
	LastUsed time.Time
}

// Recent returns recently visited aliases sorted by last_used descending
func Recent(limit int) ([]RecentEntry, error) {
	cfg, err := config.Load()
	if err != nil {
		return nil, err
	}

	db := database.New(cfg.DatabasePath)
	entries, err := db.LoadEntries()
	if err != nil {
		return nil, err
	}

	// Filter to only entries that have been used
	var usedEntries []database.AliasEntry
	for _, entry := range entries {
		if !entry.LastUsed.IsZero() {
			usedEntries = append(usedEntries, entry)
		}
	}

	if len(usedEntries) == 0 {
		return []RecentEntry{}, nil
	}

	// Sort by last_used descending
	sort.Slice(usedEntries, func(i, j int) bool {
		return usedEntries[i].LastUsed.After(usedEntries[j].LastUsed)
	})

	// Limit results
	if limit > 0 && limit < len(usedEntries) {
		usedEntries = usedEntries[:limit]
	}

	// Convert to RecentEntry
	result := make([]RecentEntry, len(usedEntries))
	for i, entry := range usedEntries {
		result[i] = RecentEntry{
			Alias:    entry.Name,
			Path:     entry.Path,
			LastUsed: entry.LastUsed,
		}
	}

	return result, nil
}

// ShowRecent displays recently visited aliases
func ShowRecent(limit int) error {
	if limit <= 0 {
		limit = 10 // Default limit
	}

	entries, err := Recent(limit)
	if err != nil {
		return err
	}

	if len(entries) == 0 {
		fmt.Println("No recently visited directories")
		return nil
	}

	fmt.Println("Recently Visited:")
	w := tabwriter.NewWriter(os.Stdout, 0, 0, 4, ' ', 0)
	for i, entry := range entries {
		timeAgo := formatTimeAgo(entry.LastUsed)
		fmt.Fprintf(w, "  %d.\t%s\t%s\t(%s)\n", i+1, entry.Alias, entry.Path, timeAgo)
	}
	w.Flush()

	return nil
}

// NavigateToRecent navigates to the Nth most recent alias
func NavigateToRecent(index int) error {
	entries, err := Recent(0) // Get all recent entries
	if err != nil {
		return err
	}

	if len(entries) == 0 {
		return fmt.Errorf("no recently visited directories")
	}

	if index < 1 || index > len(entries) {
		return fmt.Errorf("invalid recent index: %d (valid: 1-%d)", index, len(entries))
	}

	// Navigate to the alias
	return Navigate(entries[index-1].Alias)
}

// ClearRecent clears the last_used timestamps for all aliases
func ClearRecent() error {
	cfg, err := config.Load()
	if err != nil {
		return err
	}

	db := database.New(cfg.DatabasePath)
	if err := db.ClearRecentHistory(); err != nil {
		return err
	}

	fmt.Println("Cleared recent history")
	return nil
}
