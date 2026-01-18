package stack

import (
	"os"
	"path/filepath"
	"testing"
)

func TestStack(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "goto-stack-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	stackPath := filepath.Join(tmpDir, "stack")
	s := New(stackPath)

	// Test empty stack
	_, err = s.Pop()
	if err != ErrEmptyStack {
		t.Errorf("Expected ErrEmptyStack, got %v", err)
	}

	// Test push and pop
	s.Push("/first")
	s.Push("/second")
	s.Push("/third")

	size, _ := s.Size()
	if size != 3 {
		t.Errorf("Expected size 3, got %d", size)
	}

	// Pop should return in LIFO order
	val, _ := s.Pop()
	if val != "/third" {
		t.Errorf("Expected /third, got %s", val)
	}

	val, _ = s.Pop()
	if val != "/second" {
		t.Errorf("Expected /second, got %s", val)
	}

	val, _ = s.Pop()
	if val != "/first" {
		t.Errorf("Expected /first, got %s", val)
	}

	// Should be empty now
	_, err = s.Pop()
	if err != ErrEmptyStack {
		t.Errorf("Expected ErrEmptyStack, got %v", err)
	}
}
