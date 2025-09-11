# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build Commands
- `cargo build` - Build the project
- `cargo check` - Check code without building
- `cargo clippy` - Run clippy lints (strict pedantic + nursery warnings enabled)
- `cargo fmt` - Format code

### Testing
- `cargo test` - Run all tests

### Running
- `cargo run -- --last` - Run and restore last session
- `cargo run -- --url mongodb://localhost:27017` - Run with direct connection
- `cargo run -- --connection local` - Run with saved connection

### Development Tools
- `just start-mongo` - Start MongoDB container with test data using docker-compose
- `just logs` - Tail application logs (~/.local/share/tongo/tongo.log)
- `just record-demo` - Record demo using VHS

## Architecture Overview

**tongo** is a TUI (Terminal User Interface) MongoDB client built with Rust. The architecture follows a component-based design with an event-driven system.

### Core Architecture

- **App (`src/app.rs`)**: Main application orchestrator managing tabs, event processing, and rendering
- **Component System (`src/components/`)**: All UI components implement the `Component` trait with standardized command/event handling
- **System Layer (`src/system/`)**: Event-driven architecture with Commands, Events, and Messages
  - **Commands**: User intentions from keypresses (configurable)
  - **Events**: Internal program events broadcast to all components
  - **Messages**: Direct component-to-component communication

### Key Modules

- **Model (`src/model/`)**: Data structures for connections, databases, and collections
- **Client (`src/client.rs`)**: MongoDB client wrapper and async operations
- **Config (`src/config/`)**: Configuration management including key mappings and color themes
- **Persistence (`src/persistence.rs`)**: Session state saving/loading
- **Utils (`src/utils/`)**: Clipboard, storage, document editing, fuzzy search utilities

### Component Hierarchy

```
App
├── TabBar (when multiple tabs)
├── Tab (current tab)
│   ├── ConnectionScreen (connection selection)
│   └── PrimaryScreen (data browsing)
│       ├── DatabaseList
│       ├── CollectionList  
│       ├── Documents (main data view)
│       └── FilterInput
├── StatusBar
└── HelpModal (overlay)
```

### Configuration & Theming

- Default config created at `~/.config/tongo/config.toml` (or Windows equivalent)
- Custom themes via `theme.toml` in config directory
- All keybindings are fully configurable
- Page size and other UI settings customizable

### Data Flow

1. Raw terminal events → Commands (via key mappings)
2. Commands processed by App → distributed to focused components
3. Components emit Events/Messages during processing
4. Events broadcast to all visible components
5. Messages sent directly to target components
6. Async MongoDB operations queued and executed per-tab
7. Results trigger Events to update UI

### Testing & Development

- Uses standard Rust testing with `cargo test`
- Mock storage available in `src/testing/`
- Logging configured via `TONGO_LOGLEVEL` environment variable
- Docker Compose setup for local MongoDB development
