# Personal Relationship Manager

A CLI tool for tracking relationships, interactions, and reminders to stay in touch.

Currently a prototype with basic functionality for proof of concept.

## Quick Start

```bash
sbt run
```

Data is stored at `~/.relationships/network.json`.

## Basic Commands

| Command | Description |
|---------|-------------|
| `list` | List all people |
| `add <n>` | Add a person |
| `show-person <n>` | Show person details |
| `edit <n>` | Edit a person |
| `log <n>` | Log an interaction |
| `remind` | Show overdue reminders |
| `circles` | List circles |
| `labels` | List labels |
| `stats` | Show statistics |
| `help` | Show all commands |

## Code Structure

```
src/main/scala/network/
├── Model.scala          # Data types (Person, Circle, Interaction, etc.)
├── NetworkOps.scala     # Operations (add, update, delete)
├── NetworkQueries.scala # Queries (find, filter, stats)
├── JsonCodecs.scala     # JSON serialization
├── Validation.scala     # Input validation
├── CLI.scala            # Main entry point and REPL
├── PersonCommands.scala # Person CLI commands
├── CircleCommands.scala # Circle CLI commands
└── InteractionCommands.scala # Interaction/reminder/label commands
```

## Building

```bash
# Run tests
sbt test

# Build JAR
sbt assembly
java -jar target/scala-3.7.4/relationships.jar
```

## Example Session

```
> add Guy Testadopoulos
Added Guy Testadopoulos
How did you meet? Work conference
Labels (enter numbers separated by spaces, or press Enter to skip):
  1. acquaintance
  2. coworker
  3. family
  4. friend
Labels: 2 4
Reminder every how many days? 14
Reminder set for every 14 days

> show-person Guy Testadopoulos
Name: Guy Testadopoulos
Nickname: (none)
Birthday: (none)
How we met: Work conference
Notes: (none)
Default location: (none)
Labels: coworker, friend
Circles: (none)
Phones: (none)
Emails: (none)
Reminder: every 14 days
Last interaction: (never)
Total interactions: 0

> log Guy Testadopoulos
Logging interaction with Guy Testadopoulos
How did you interact?
  1. In Person
  2. Text
  3. Phone Call
  4. Video Call
  5. Social Media
Medium (1-5): 1
Location: Coffee shop
Topics (comma-separated): catch up, work
Note (optional): 
Logged interaction with Guy Testadopoulos

> remind
No overdue reminders! You're all caught up.
```
