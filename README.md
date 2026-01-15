# Personal Relationship Manager

A CLI tool for tracking relationships, interactions, and reminders to stay in touch. This is a working prototype for proof of concept.

## Setup

1. Install Java 17+ ([download](https://adoptium.net/))
2. Install sbt ([download](https://www.scala-sbt.org/download.html))
3. Clone and run:

```bash
git clone https://github.com/petros-p/prm.git
cd prm
sbt run
```

Data is stored at `.data/network.json` within the project directory.

## Commands

| Command | Description |
|---------|-------------|
| `list` | List all people |
| `add <name>` | Add a person |
| `show-person <name>` | Show person details |
| `edit <name>` | Edit a person |
| `log <name>` | Log an interaction |
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
java -jar target/scala-3.7.4/prm.jar
```

## Future Work

- [ ] Web app with REST API
- [ ] Mobile app
- [ ] Import contacts from phone/email
- [ ] Automated interaction logging (email, calendar, texts)
- [ ] Birthday reminders
- [ ] Relationship strength scoring
- [ ] Data export (CSV, vCard)
- [ ] Multi-device sync

## Example Session

```
> add Guy Testadopoulos
Added Guy Testadopoulos
How did you meet? Work conference
Labels: 2 4
Reminder every how many days? 14
Reminder set for every 14 days

> show-person Guy
Name: Guy Testadopoulos
Nickname: (none)
Birthday: (none)
How we met: Work conference
Labels: coworker, friend
Reminder: every 14 days
Last interaction: (never)

> log Guy
Medium (1-5): 1
Location: Coffee shop
Topics (comma-separated): catch up, work
Logged interaction with Guy Testadopoulos
```
