# Cortex File Manager

## Project Overview

Cortex is a modern, cross-platform, and keyboard-driven orthodox file manager written in Rust. It is inspired by Far Manager and features a dual-panel interface, a clean terminal UI, and an integrated AI assistant. The project is built with a modular architecture, with separate crates for core functionality, the TUI, the CLI, plugins, and platform-specific code.

**Key Technologies:**

*   **Language:** Rust
*   **UI:** Ratatui (a TUI library for Rust)
*   **Plugin System:** Lua
*   **Package Manager:** Cargo

**Architecture:**

The project is a Rust workspace with the following crates:

*   `cortex-core`: Core functionalities like file system operations, state management, AI integration, caching, and more.
*   `cortex-tui`: Terminal UI components and dialogs.
*   `cortex-cli`: The main application entry point and command-line argument parsing.
*   `cortex-plugins`: The Lua plugin system.
*   `cortex-platform`: Platform-specific functionalities like clipboard access and trash management.
*   `cortex-updater`: Handles application updates.

## Building and Running

**Prerequisites:**

*   Rust 1.70 or higher
*   Cargo

**Building:**

*   **Debug build:**
    ```bash
    cargo build
    ```
*   **Release build (optimized):**
    ```bash
    cargo build --release
    ```

**Running:**

*   **Run the application:**
    ```bash
    ./target/release/cortex
    ```
*   **Run with logging:**
    ```bash
    RUST_LOG=debug cargo run
    ```

**Testing:**

*   **Run tests:**
    ```bash
    cargo test
    ```

## Development Conventions

*   **Coding Style:** The project follows standard Rust conventions.
*   **Testing:** The project has a suite of tests that can be run with `cargo test`.
*   **Contributions:** Contributions are welcome and can be submitted as pull requests.
