mod todo_manager;

use anyhow::Result;
use rmcp::{
    transport::stdio,
    ServiceExt,
    handler::{server::tool::Parameters, server::ServerHandler},
    model::{ServerCapabilities, ServerInfo, ProtocolVersion, Implementation, ListToolsResult, CallToolResult, Tool, PaginatedRequestParam, CallToolRequestParam, Content},
    service::{RoleServer, RequestContext},
    Error as McpError,
};
use std::sync::Arc;
use serde_json::json;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;
use crate::todo_manager::{MarkdownTodoManager, TodoStatus};
use std::borrow::Cow;

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

#[async_trait]
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

    fn list_tools(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        let tools = vec![
            Tool {
                name: "add_todo".into(),
                description: Cow::Borrowed("Add a new TODO item"),
                input_schema: Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "text": { "type": "string" }
                    },
                    "required": ["text"]
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "remove_todo".into(),
                description: Cow::Borrowed("Remove a TODO item by index"),
                input_schema: Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "index": { "type": "integer" }
                    },
                    "required": ["index"]
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "list_todos".into(),
                description: Cow::Borrowed("List all TODO items"),
                input_schema: Arc::new(json!({
                    "type": "object",
                    "properties": {}
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "mark_done".into(),
                description: Cow::Borrowed("Mark a TODO item as done"),
                input_schema: Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "index": { "type": "integer" }
                    },
                    "required": ["index"]
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "mark_in_progress".into(),
                description: Cow::Borrowed("Mark a TODO item as in progress"),
                input_schema: Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "index": { "type": "integer" }
                    },
                    "required": ["index"]
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "get_in_progress_index".into(),
                description: Cow::Borrowed("Get the index of the currently in progress TODO item"),
                input_schema: Arc::new(json!({
                    "type": "object",
                    "properties": {}
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "unmark_done".into(),
                description: Cow::Borrowed("Unmark a TODO item (mark as not done)"),
                input_schema: Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "index": { "type": "integer" }
                    },
                    "required": ["index"]
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "find_next".into(),
                description: Cow::Borrowed("Find the next incomplete TODO item"),
                input_schema: Arc::new(json!({
                    "type": "object",
                    "properties": {}
                }).as_object().unwrap().clone()),
            },
            Tool {
                name: "add_comment".into(),
                description: Cow::Borrowed("Add a comment to a TODO item"),
                input_schema: Arc::new(json!({
                    "type": "object",
                    "properties": {
                        "index": { "type": "integer" },
                        "comment": { "type": "string" }
                    },
                    "required": ["index", "comment"]
                }).as_object().unwrap().clone()),
            },
        ];
        std::future::ready(Ok(ListToolsResult {
            tools,
            next_cursor: None,
        }))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        let this = self.clone();
        async move {
            match request.name.as_ref() {
                "add_todo" => {
                    let args_obj = request.arguments.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
                    let args: AddTodoArgs = serde_json::from_value(serde_json::Value::Object(args_obj))
                        .map_err(|e| McpError::invalid_params(format!("Invalid params: {}", e), None))?;
                    let res = this.add_todo(Parameters(args)).await
                        .map_err(|e| McpError::internal_error(e, None))?;
                    Ok(CallToolResult::success(vec![Content::text(res)]))
                }
                "remove_todo" => {
                    let args_obj = request.arguments.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
                    let args: IndexArgs = serde_json::from_value(serde_json::Value::Object(args_obj))
                        .map_err(|e| McpError::invalid_params(format!("Invalid params: {}", e), None))?;
                    let res = this.remove_todo(Parameters(args)).await
                        .map_err(|e| McpError::internal_error(e, None))?;
                    Ok(CallToolResult::success(vec![Content::text(res)]))
                }
                "list_todos" => {
                    let res = this.list_todos().await
                        .map_err(|e| McpError::internal_error(e, None))?;
                    Ok(CallToolResult::success(vec![Content::text(res)]))
                }
                "mark_done" => {
                    let args_obj = request.arguments.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
                    let args: IndexArgs = serde_json::from_value(serde_json::Value::Object(args_obj))
                        .map_err(|e| McpError::invalid_params(format!("Invalid params: {}", e), None))?;
                    let res = this.mark_done(Parameters(args)).await
                        .map_err(|e| McpError::internal_error(e, None))?;
                    Ok(CallToolResult::success(vec![Content::text(res)]))
                }
                "mark_in_progress" => {
                    let args_obj = request.arguments.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
                    let args: IndexArgs = serde_json::from_value(serde_json::Value::Object(args_obj))
                        .map_err(|e| McpError::invalid_params(format!("Invalid params: {}", e), None))?;
                    let res = this.mark_in_progress(Parameters(args)).await
                        .map_err(|e| McpError::internal_error(e, None))?;
                    Ok(CallToolResult::success(vec![Content::text(res)]))
                }
                "get_in_progress_index" => {
                    let res = this.get_in_progress_index().await
                        .map_err(|e| McpError::internal_error(e, None))?;
                    Ok(CallToolResult::success(vec![Content::text(res)]))
                }
                "unmark_done" => {
                    let args_obj = request.arguments.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
                    let args: IndexArgs = serde_json::from_value(serde_json::Value::Object(args_obj))
                        .map_err(|e| McpError::invalid_params(format!("Invalid params: {}", e), None))?;
                    let res = this.unmark_done(Parameters(args)).await
                        .map_err(|e| McpError::internal_error(e, None))?;
                    Ok(CallToolResult::success(vec![Content::text(res)]))
                }
                "find_next" => {
                    let res = this.find_next().await
                        .map_err(|e| McpError::internal_error(e, None))?;
                    Ok(CallToolResult::success(vec![Content::text(res)]))
                }
                "add_comment" => {
                    let args_obj = request.arguments.ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;
                    let args: CommentArgs = serde_json::from_value(serde_json::Value::Object(args_obj))
                        .map_err(|e| McpError::invalid_params(format!("Invalid params: {}", e), None))?;
                    let res = this.add_comment(Parameters(args)).await
                        .map_err(|e| McpError::internal_error(e, None))?;
                    Ok(CallToolResult::success(vec![Content::text(res)]))
                }
                _ => Err(McpError::method_not_found::<rmcp::model::CallToolRequestMethod>()),
            }
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
