mod todo_manager;

use anyhow::Result;
use rmcp::{
    transport::stdio,
    ServiceExt,
    tool,
    handler::{server::tool::Parameters, server::ServerHandler},
    model::{ServerCapabilities, ServerInfo, ProtocolVersion, Implementation},
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;
use crate::todo_manager::{MarkdownTodoManager, TodoStatus};

#[derive(Clone)]
struct TodoServer {
    #[allow(dead_code)]
    manager: std::sync::Arc<MarkdownTodoManager>,
}

#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct AddTodoArgs {
    text: String,
}

#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct IndexArgs {
    index: usize,
}

#[allow(dead_code)]
#[derive(Deserialize, JsonSchema)]
struct CommentArgs {
    index: usize,
    comment: String,
}

#[tool(tool_box)]
impl TodoServer {
    /// Add a new TODO item
    async fn add_todo(&self, Parameters(args): Parameters<AddTodoArgs>) -> Result<String, String> {
        self.manager.add_todo(&args.text).map_err(|e| e.to_string())?;
        Ok(format!("Added TODO: {}", args.text))
    }

    /// Remove a TODO item by index
    async fn remove_todo(&self, Parameters(args): Parameters<IndexArgs>) -> Result<String, String> {
        self.manager.remove_todo(args.index).map_err(|e| e.to_string())?;
        Ok(format!("Removed TODO at index {}", args.index))
    }

    /// List all TODO items
    async fn list_todos(&self) -> Result<String, String> {
        let (_, todos) = self.manager.read_todos().map_err(|e| e.to_string())?;
        if todos.is_empty() {
            return Ok("No TODO items found.".to_string());
        }
        let mut output = String::new();
        for (i, todo) in todos.iter().enumerate() {
            let status = match todo.status {
                TodoStatus::Todo => "[ ]",
                TodoStatus::InProgress => "[/]",
                TodoStatus::Done => "[x]",
            };
            output.push_str(&format!("{}: {} {}\n", i, status, todo.text));
        }
        Ok(output)
    }

    /// Mark a TODO item as done
    async fn mark_done(&self, Parameters(args): Parameters<IndexArgs>) -> Result<String, String> {
        self.manager.set_status(args.index, TodoStatus::Done).map_err(|e| e.to_string())?;
        Ok(format!("Marked TODO {} as done", args.index))
    }

    /// Mark a TODO item as in progress
    async fn mark_in_progress(&self, Parameters(args): Parameters<IndexArgs>) -> Result<String, String> {
        self.manager.set_status(args.index, TodoStatus::InProgress).map_err(|e| e.to_string())?;
        Ok(format!("Marked TODO {} as in progress", args.index))
    }

    /// Get the index of the currently in progress TODO item
    async fn get_in_progress_index(&self) -> Result<String, String> {
        let (_, todos) = self.manager.read_todos().map_err(|e| e.to_string())?;
        for (i, todo) in todos.iter().enumerate() {
            if todo.status == TodoStatus::InProgress {
                return Ok(format!("{}", i));
            }
        }
        Err("No TODO item is currently in progress.".to_string())
    }

    /// Unmark a TODO item (mark as not done)
    async fn unmark_done(&self, Parameters(args): Parameters<IndexArgs>) -> Result<String, String> {
        self.manager.set_status(args.index, TodoStatus::Todo).map_err(|e| e.to_string())?;
        Ok(format!("Unmarked TODO {}", args.index))
    }

    /// Find the next incomplete TODO item
    async fn find_next(&self) -> Result<String, String> {
        let (_, todos) = self.manager.read_todos().map_err(|e| e.to_string())?;
        for (i, todo) in todos.iter().enumerate() {
            if todo.status != TodoStatus::Done {
                return Ok(format!("Next TODO (index {}): {}", i, todo.text));
            }
        }
        Ok("No incomplete TODO items found.".to_string())
    }

    /// Add a comment to a TODO item
    async fn add_comment(&self, Parameters(args): Parameters<CommentArgs>) -> Result<String, String> {
        self.manager.add_comment(args.index, &args.comment).map_err(|e| e.to_string())?;
        Ok(format!("Added comment to TODO {}", args.index))
    }
}

#[async_trait::async_trait]
impl ServerHandler for TodoServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "md-todo-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some("A Markdown TODO manager MCP server".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Redirect tracing to stderr so it doesn't interfere with stdio transport
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let todo_file = if args.len() > 1 {
        args[1].clone()
    } else {
        std::env::var("TODO_FILE").unwrap_or_else(|_| "todo.md".to_string())
    };

    let manager = MarkdownTodoManager::new(PathBuf::from(todo_file));
    let server = TodoServer {
        manager: std::sync::Arc::new(manager),
    };

    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
