# CLAUDE.md

This file provides guidance to Claude Code when working with the Rust PRM codebase.

## Project Overview

Personal Relationship Manager (PRM) — a Rust CLI tool for tracking relationships, interactions, and reminders. Rewrite of the Scala prototype with architectural improvements: SQLite persistence, local-only AI, privacy-first design.

## Design Priorities

- **Privacy & security** — All data stays local. No external API calls. AI runs locally via Ollama.
- **Speed** — Rust for performance. SQLite for fast queries.
- **Ease of use** — Interactive REPL with partial-match name search, sensible defaults.

## Build & Test Commands

```bash
cargo run                          # Run the interactive CLI (DB at .data/prm.db)
cargo run -- --file path/to/db     # Custom database path
cargo run -- --import old.json     # Import Scala PRM JSON data
cargo run -- --help                # Show CLI flags
cargo test                         # Run all tests (99 tests, 5 suites)
cargo test --test db_tests         # Run a single test file
cargo test test_name               # Run tests matching a name
cargo build --release              # Build optimized binary
cargo check                        # Fast compile check (no codegen)
```

## Architecture

Source is in `src/`. Strict separation of concerns across layers:

### Domain Layer
- **`model/`** — Domain types (structs, enums). Type-safe IDs via `Id<T>` with PhantomData. Key entities: `Person`, `Relationship`, `Interaction`, `Circle`, `RelationshipLabel`, `ContactEntry`.
- **`error.rs`** — `PrmError` enum with `thiserror`. All operations return `PrmResult<T>`.
- **`validation.rs`** — Input validation returning `PrmResult<T>`.

### Persistence Layer
- **`db/schema.rs`** — SQLite schema (11 tables). `initialize()` creates tables idempotently.
- **`db/*_repo.rs`** — Repository modules: `person_repo`, `contact_repo`, `relationship_repo`, `interaction_repo`, `circle_repo`, `network_repo`. Raw SQL via rusqlite.

### Business Logic Layer
- **`ops/`** — Write operations. Each returns `PrmResult<T>`. Modules: `person_ops`, `contact_ops`, `relationship_ops`, `interaction_ops`, `circle_ops`, `label_ops`.
- **`queries/`** — Read-only queries. Modules: `person_queries`, `relationship_queries`, `interaction_queries`, `circle_queries`, `contact_queries`, `reminder_queries`, `stats_queries`.

### AI Layer
- **`ai/llm_service.rs`** — Ollama client for natural language interaction parsing. Runs locally, no external API calls. Default model: `llama3.2:3b`. Configurable via `OLLAMA_HOST` and `PRM_MODEL` env vars.

### CLI Layer
- **`cli/mod.rs`** — REPL loop, command dispatch, network initialization.
- **`cli/context.rs`** — `CLIContext` holds DB connection, user, self_id. Helper methods for prompting, person/circle/label lookup.
- **`cli/person_commands.rs`** — Person CRUD and interactive edit flows.
- **`cli/circle_commands.rs`** / **`label_commands.rs`** / **`interaction_commands.rs`** — Command handlers.
- **`cli/ai_log_command.rs`** — AI-assisted interaction logging with review/edit/save flow.

### Migration
- **`migrate/mod.rs`** — Imports Scala PRM JSON format into SQLite. Invoked via `--import` flag.

## Key Design Patterns

- **Result-based error handling** — All operations return `PrmResult<T>`, never panic.
- **Owned Connection in CLIContext** — `CLIContext` owns the `rusqlite::Connection`. Repos borrow it via `&Connection`.
- **Normalized DB vs embedded Scala model** — Scala embedded contacts in Person and interactions in Relationship. Rust normalizes into separate tables.
- **No async** — Blocking I/O throughout. Ollama calls use `ureq` (blocking HTTP).

## Dependencies

- **Rust 1.93+** (stable)
- **rusqlite 0.31** (bundled SQLite) — persistence
- **chrono 0.4** — dates
- **serde + serde_json** — serialization
- **uuid 1** — ID generation
- **ureq 2** — HTTP client for Ollama
- **thiserror 1** — error derive

## Test Structure

Tests are in `tests/`:
- `model_tests.rs` (24) — Model creation, ID type safety, interaction factories
- `db_tests.rs` (19) — Repository CRUD, schema initialization
- `ops_tests.rs` (25) — Business operations, validation, error cases
- `queries_tests.rs` (12) — Query functions, reminders, stats
- `lib.rs` unit tests (19) — Validation, ID parsing
