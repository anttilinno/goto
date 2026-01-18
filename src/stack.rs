//! Directory stack for push/pop navigation

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur during stack operations
#[derive(Error, Debug)]
pub enum StackError {
    #[error("directory stack is empty")]
    Empty,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Directory stack for push/pop operations
pub struct Stack {
    path: PathBuf,
}

impl Stack {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Push a directory onto the stack
    pub fn push(&self, dir: &str) -> Result<(), StackError> {
        let mut entries = self.load()?;
        entries.push(dir.to_string());
        self.save(&entries)
    }

    /// Pop and return the top directory from the stack
    pub fn pop(&self) -> Result<String, StackError> {
        let mut entries = self.load()?;

        if entries.is_empty() {
            return Err(StackError::Empty);
        }

        let dir = entries.pop().unwrap();
        self.save(&entries)?;
        Ok(dir)
    }

    /// Peek at the top directory without removing it
    pub fn peek(&self) -> Result<String, StackError> {
        let entries = self.load()?;

        entries.last().cloned().ok_or(StackError::Empty)
    }

    /// Get the number of entries in the stack
    pub fn size(&self) -> Result<usize, StackError> {
        Ok(self.load()?.len())
    }

    /// Clear all entries from the stack
    pub fn clear(&self) -> Result<(), StackError> {
        self.save(&[])
    }

    fn load(&self) -> Result<Vec<String>, StackError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                entries.push(trimmed.to_string());
            }
        }

        Ok(entries)
    }

    fn save(&self, entries: &[String]) -> Result<(), StackError> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&self.path)?;
        for entry in entries {
            writeln!(file, "{}", entry)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_push_pop() {
        let dir = tempdir().unwrap();
        let stack = Stack::new(dir.path().join("stack"));

        stack.push("/home/user/a").unwrap();
        stack.push("/home/user/b").unwrap();

        assert_eq!(stack.size().unwrap(), 2);
        assert_eq!(stack.pop().unwrap(), "/home/user/b");
        assert_eq!(stack.pop().unwrap(), "/home/user/a");
        assert!(matches!(stack.pop(), Err(StackError::Empty)));
    }

    #[test]
    fn test_peek() {
        let dir = tempdir().unwrap();
        let stack = Stack::new(dir.path().join("stack"));

        stack.push("/home/user").unwrap();
        assert_eq!(stack.peek().unwrap(), "/home/user");
        assert_eq!(stack.size().unwrap(), 1); // Still there
    }

    #[test]
    fn test_clear() {
        let dir = tempdir().unwrap();
        let stack = Stack::new(dir.path().join("stack"));

        stack.push("/a").unwrap();
        stack.push("/b").unwrap();
        stack.clear().unwrap();
        assert_eq!(stack.size().unwrap(), 0);
    }

    #[test]
    fn test_empty_stack_errors() {
        let dir = tempdir().unwrap();
        let stack = Stack::new(dir.path().join("stack"));

        assert!(matches!(stack.pop(), Err(StackError::Empty)));
        assert!(matches!(stack.peek(), Err(StackError::Empty)));
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();
        let stack_path = dir.path().join("stack");

        // Push with one instance
        {
            let stack = Stack::new(stack_path.clone());
            stack.push("/first").unwrap();
            stack.push("/second").unwrap();
        }

        // Read with another instance
        {
            let stack = Stack::new(stack_path);
            assert_eq!(stack.size().unwrap(), 2);
            assert_eq!(stack.peek().unwrap(), "/second");
        }
    }
}
