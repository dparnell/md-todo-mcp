# Markdown TODO MCP Server

A Model Context Protocol (MCP) server that allows AI models to manage a TODO list stored in a Markdown file.

## Features

- **Add TODO**: Create new tasks in the markdown file.
- **List TODOs**: View all existing tasks with their completion status.
- **Mark/Unmark Done/In Progress**: Toggle the completion status or set tasks as in progress.
- **Remove TODO**: Delete tasks from the list.
- **Add Comments**: Add sub-bullet comments to specific tasks.
- **Find Next**: Quickly identify the next incomplete task.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition or later)

### Build

```bash
cargo build --release
```

The binary will be located at `target/release/md-todo-mcp`.

## Usage

### Environment Variables

- `TODO_FILE`: Path to the markdown file used for storing tasks. Defaults to `todo.md` in the current directory if not specified.

### Running the Server

The server uses standard input/output (stdio) for communication.

```bash
# Using default todo.md
./target/release/md-todo-mcp

# Using a custom file (command line argument)
./target/release/md-todo-mcp my_tasks.md

# Using a custom file (environment variable)
TODO_FILE="my_tasks.md" ./target/release/md-todo-mcp
```

### Integration with MCP Clients (e.g., Claude Desktop)

To use this server with Claude Desktop, add the following to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "md-todo": {
      "command": "path/to/your/md-todo-mcp",
      "args": ["path/to/your/todo.md"]
    }
  }
}
```

Alternatively, you can use the `env` field as before:

```json
{
  "mcpServers": {
    "md-todo": {
      "command": "path/to/your/md-todo-mcp",
      "env": {
        "TODO_FILE": "path/to/your/todo.md"
      }
    }
  }
}
```

## Markdown Format

The server expects and maintains tasks in the following standard markdown format:

```markdown
# TODO List
- [ ] Task 1
  - Comment: Something to remember
- [/] In Progress Task
- [x] Completed Task
```

## Tools Provided

- `add_todo(text: String)`: Adds a new TODO item.
- `list_todos()`: Lists all TODO items.
- `mark_done(index: usize)`: Marks a TODO item as done.
- `mark_in_progress(index: usize)`: Marks a TODO item as in progress.
- `get_in_progress_index()`: Returns the index of the currently in progress TODO item.
- `unmark_done(index: usize)`: Marks a TODO item as not done.
- `remove_todo(index: usize)`: Removes a TODO item.
- `add_comment(index: usize, comment: String)`: Adds a comment to a TODO item.
- `find_next()`: Finds the next incomplete TODO item.
