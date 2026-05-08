use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TodoStatus {
    Todo,
    InProgress,
    Done,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TodoItem {
    pub line_index: usize,
    pub status: TodoStatus,
    pub text: String,
}

pub struct MarkdownTodoManager {
    pub path: PathBuf,
}

impl MarkdownTodoManager {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[allow(dead_code)]
    fn ensure_file_exists(&self) -> Result<()> {
        if !self.path.exists() {
            fs::write(&self.path, "# TODO List\n")?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn read_todos(&self) -> Result<(Vec<String>, Vec<TodoItem>)> {
        self.ensure_file_exists()?;
        let content = fs::read_to_string(&self.path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut todos = Vec::new();

        // Regex for markdown task list items: - [ ] or - [x] or - [/] or * [ ] or * [x] or * [/]
        let re = Regex::new(r"^\s*[-*]\s+\[([ xX/])\]\s+(.*)$").unwrap();

        for (i, line) in lines.iter().enumerate() {
            if let Some(caps) = re.captures(line) {
                let status_char = &caps[1];
                let status = if status_char == "/" {
                    TodoStatus::InProgress
                } else if status_char.to_lowercase() == "x" {
                    TodoStatus::Done
                } else {
                    TodoStatus::Todo
                };
                let text = caps[2].to_string();
                todos.push(TodoItem {
                    line_index: i,
                    status,
                    text,
                });
            }
        }

        Ok((lines, todos))
    }

    #[allow(dead_code)]
    pub fn write_lines(&self, lines: Vec<String>) -> Result<()> {
        let mut content = lines.join("\n");
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        fs::write(&self.path, content)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn add_todo(&self, text: &String) -> Result<()> {
        let (mut lines, _) = self.read_todos()?;
        lines.push(format!("- [ ] {}", text));
        self.write_lines(lines)
    }

    #[allow(dead_code)]
    pub fn remove_todo(&self, index: usize) -> Result<()> {
        let (mut lines, todos) = self.read_todos()?;
        if index >= todos.len() {
            return Err(anyhow::anyhow!("Index out of bounds"));
        }
        let todo = &todos[index];
        lines.remove(todo.line_index);
        
        // If we added a comment as a sub-bullet right after, we might want to remove it too?
        // But the requirement just says "remove a todo item".
        // If we remove the line, subsequent indexes in 'todos' would be wrong, 
        // but we re-read anyway.
        
        self.write_lines(lines)
    }

    #[allow(dead_code)]
    pub fn set_status(&self, index: usize, status: TodoStatus) -> Result<()> {
        let (mut lines, todos) = self.read_todos()?;
        if index >= todos.len() {
            return Err(anyhow::anyhow!("Index out of bounds"));
        }
        let todo = &todos[index];
        let line = &lines[todo.line_index];
        
        let re = Regex::new(r"(\s*[-*]\s+\[)[ xX/](\]\s+.*)").unwrap();
        let mark = match status {
            TodoStatus::Todo => " ",
            TodoStatus::InProgress => "/",
            TodoStatus::Done => "x",
        };
        let new_line = re.replace(line, format!("${{1}}{}${{2}}", mark)).to_string();
        
        lines[todo.line_index] = new_line;
        self.write_lines(lines)
    }

    #[allow(dead_code)]
    pub fn set_done(&self, index: usize, done: bool) -> Result<()> {
        let status = if done { TodoStatus::Done } else { TodoStatus::Todo };
        self.set_status(index, status)
    }

    #[allow(dead_code)]
    pub fn add_comment(&self, index: usize, comment: &str) -> Result<()> {
        let (mut lines, todos) = self.read_todos()?;
        if index >= todos.len() {
            return Err(anyhow::anyhow!("Index out of bounds"));
        }
        let todo = &todos[index];
        // Append comment on the same line or next line? 
        // Requirements say "comment against a todo". Usually in MD this might be a sub-bullet or just text.
        // Let's do a sub-bullet.
        lines.insert(todo.line_index + 1, format!("  - Comment: {}", comment));
        self.write_lines(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_in_progress() {
        let test_file = PathBuf::from("test_todo_internal.md");
        if test_file.exists() {
            fs::remove_file(&test_file).unwrap();
        }
        
        let manager = MarkdownTodoManager::new(test_file.clone());
        manager.add_todo(&"Task 1".to_string()).unwrap();
        manager.add_todo(&"Task 2".to_string()).unwrap();
        
        manager.set_status(0, TodoStatus::InProgress).unwrap();
        
        let (_, todos) = manager.read_todos().unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].status, TodoStatus::InProgress);
        assert_eq!(todos[1].status, TodoStatus::Todo);
        
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("- [/] Task 1"));
        assert!(content.contains("- [ ] Task 2"));
        
        fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_get_in_progress() {
        let test_file = PathBuf::from("test_todo_get_in_progress.md");
        if test_file.exists() {
            fs::remove_file(&test_file).unwrap();
        }
        
        let manager = MarkdownTodoManager::new(test_file.clone());
        manager.add_todo(&"Task 1".to_string()).unwrap();
        manager.add_todo(&"Task 2".to_string()).unwrap();
        manager.set_status(1, TodoStatus::InProgress).unwrap();
        
        let (_, todos) = manager.read_todos().unwrap();
        let in_progress_idx = todos.iter().position(|t| t.status == TodoStatus::InProgress);
        assert_eq!(in_progress_idx, Some(1));
        
        fs::remove_file(&test_file).unwrap();
    }
}
