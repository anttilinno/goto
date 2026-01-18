package alias

import (
	"fmt"
	"regexp"
)

// Valid alias pattern: starts with letter/digit, followed by letters/digits/hyphens/underscores
var validAliasPattern = regexp.MustCompile(`^[a-zA-Z0-9][a-zA-Z0-9_-]*$`)

// Valid tag pattern: alphanumeric with dash/underscore, case-insensitive (stored lowercase)
var validTagPattern = regexp.MustCompile(`^[a-zA-Z0-9][a-zA-Z0-9_-]*$`)

// Alias represents a directory alias
type Alias struct {
	Name string
	Path string
}

// Validate checks if an alias name is valid
func Validate(name string) error {
	if name == "" {
		return &InvalidAliasError{Alias: name, Reason: "alias cannot be empty"}
	}

	if !validAliasPattern.MatchString(name) {
		return &InvalidAliasError{
			Alias:  name,
			Reason: "must start with letter/digit and contain only letters, digits, hyphens, underscores",
		}
	}

	return nil
}

// InvalidAliasError represents an invalid alias name error
type InvalidAliasError struct {
	Alias  string
	Reason string
}

func (e *InvalidAliasError) Error() string {
	return fmt.Sprintf("invalid alias '%s': %s", e.Alias, e.Reason)
}

// AliasNotFoundError represents a missing alias error
type AliasNotFoundError struct {
	Alias string
}

func (e *AliasNotFoundError) Error() string {
	return fmt.Sprintf("alias '%s' not found", e.Alias)
}

// AliasExistsError represents a duplicate alias error
type AliasExistsError struct {
	Alias string
}

func (e *AliasExistsError) Error() string {
	return fmt.Sprintf("alias '%s' already exists", e.Alias)
}

// DirectoryNotFoundError represents a missing directory error
type DirectoryNotFoundError struct {
	Path string
}

func (e *DirectoryNotFoundError) Error() string {
	return fmt.Sprintf("directory does not exist: %s", e.Path)
}

// InvalidTagError represents an invalid tag name error
type InvalidTagError struct {
	Tag    string
	Reason string
}

func (e *InvalidTagError) Error() string {
	return fmt.Sprintf("invalid tag '%s': %s", e.Tag, e.Reason)
}

// ValidateTag checks if a tag name is valid
func ValidateTag(tag string) error {
	if tag == "" {
		return &InvalidTagError{Tag: tag, Reason: "tag cannot be empty"}
	}

	if !validTagPattern.MatchString(tag) {
		return &InvalidTagError{
			Tag:    tag,
			Reason: "must start with letter/digit and contain only letters, digits, hyphens, underscores",
		}
	}

	return nil
}
