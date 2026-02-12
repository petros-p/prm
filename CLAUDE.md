# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Personal Relationship Manager (PRM) — a Scala 3 CLI tool for tracking relationships, interactions, and reminders. Prototype/proof of concept. Uses SBT as the build system.

## Build & Test Commands

```bash
sbt run                  # Run the interactive CLI
sbt run -- --file path   # Run with custom data file (default: .data/network.json)
sbt test                 # Run all tests (57 tests, 4 suites)
sbt "testOnly network.ModelSpec"  # Run a single test class
sbt "testOnly network.ModelSpec -- -z 'test name substring'"  # Run a single test
sbt assembly             # Build fat JAR
sbt clean                # Clean build artifacts
```

## Architecture

All source is in `src/main/scala/network/`. The codebase follows a strict separation of concerns:

- **Model.scala** — Domain types (case classes, sealed traits). Type-safe IDs via `Id[A]` generic wrapper (prevents mixing Person IDs with Circle IDs). Key entities: `Network`, `Person`, `Relationship`, `Interaction`, `Circle`, `RelationshipLabel`.
- **NetworkOps.scala** — Pure business logic. All mutations return `Either[ValidationError, A]`. Immutable updates via `copy()`. No side effects.
- **NetworkQueries.scala** — Read-only query/analytics functions. Returns filtered views, reminder statuses, statistics. No mutation.
- **Validation.scala** — Input validation helpers returning `Either[ValidationError, A]`.
- **JsonCodecs.scala** — upickle serialization with custom codecs for sealed traits. Handles backward compatibility. Provides `saveToFile`/`loadFromFile`.
- **CLI.scala** — REPL loop, command dispatching, `CLIContext` (mutable state holder), session initialization.
- **PersonCommands.scala** — Person CRUD and interactive edit flows (largest module).
- **CircleCommands.scala** / **LabelCommands.scala** / **InteractionCommands.scala** — Command handlers for their respective domains.

## Key Design Patterns

- **Either-based error handling** — Operations return `Either[ValidationError, A]`, not exceptions. Propagate errors through for-comprehensions.
- **Immutable data** — All domain objects are case classes updated via `copy()` and collection transforms.
- **CLIContext.withSave()** — Wraps any operation that modifies the network, ensuring the state is persisted to JSON after each change.
- **Sealed trait ADTs** — `InteractionMedium`, `ContactType`, `ReminderStatus`, `OverdueStatus` are sealed traits with exhaustive pattern matching.

## Dependencies

- **Scala 3.7.4**, **SBT 1.12.0**
- **upickle 4.1.0** — JSON serialization
- **scalatest 3.2.19** — Testing (FunSuite + Matchers style)
- **sbt-assembly 2.2.0** — Fat JAR packaging

## Test Structure

Tests are in `src/test/scala/network/`:
- `ModelSpec` — Model creation, ID type safety
- `NetworkOpsSpec` — Business logic operations
- `NetworkQueriesSpec` — Query and analytics functions
- `JsonCodecsSpec` — Serialization round-trips, backward compatibility
