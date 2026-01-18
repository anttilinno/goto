package stack

import (
	"bufio"
	"errors"
	"fmt"
	"os"
	"strings"
)

var ErrEmptyStack = errors.New("directory stack is empty")

// Stack handles the directory stack for push/pop operations
type Stack struct {
	path string
}

// New creates a new Stack instance
func New(path string) *Stack {
	return &Stack{path: path}
}

// Push adds a directory to the top of the stack
func (s *Stack) Push(dir string) error {
	// Read existing stack
	entries, err := s.load()
	if err != nil {
		return err
	}

	entries = append(entries, dir)
	return s.save(entries)
}

// Pop removes and returns the top directory from the stack
func (s *Stack) Pop() (string, error) {
	entries, err := s.load()
	if err != nil {
		return "", err
	}

	if len(entries) == 0 {
		return "", ErrEmptyStack
	}

	// Get last entry (top of stack)
	dir := entries[len(entries)-1]
	entries = entries[:len(entries)-1]

	if err := s.save(entries); err != nil {
		return "", err
	}

	return dir, nil
}

// Peek returns the top directory without removing it
func (s *Stack) Peek() (string, error) {
	entries, err := s.load()
	if err != nil {
		return "", err
	}

	if len(entries) == 0 {
		return "", ErrEmptyStack
	}

	return entries[len(entries)-1], nil
}

// Size returns the number of entries in the stack
func (s *Stack) Size() (int, error) {
	entries, err := s.load()
	if err != nil {
		return 0, err
	}
	return len(entries), nil
}

// Clear removes all entries from the stack
func (s *Stack) Clear() error {
	return s.save([]string{})
}

func (s *Stack) load() ([]string, error) {
	file, err := os.Open(s.path)
	if os.IsNotExist(err) {
		return []string{}, nil
	}
	if err != nil {
		return nil, err
	}
	defer file.Close()

	var entries []string
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line != "" {
			entries = append(entries, line)
		}
	}

	return entries, scanner.Err()
}

func (s *Stack) save(entries []string) error {
	file, err := os.Create(s.path)
	if err != nil {
		return err
	}
	defer file.Close()

	for _, entry := range entries {
		if _, err := fmt.Fprintln(file, entry); err != nil {
			return err
		}
	}

	return nil
}
