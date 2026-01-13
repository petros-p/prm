# Personal Relationship Manager

A command-line tool for tracking your relationships, including with yourself. Keep notes on people you know, log interactions, and get reminders to stay in touch.

## Features

- **Track people** in your network with notes, labels, and contact info (including yourself)
- **Log interactions** with medium (In Person, Text, Phone Call, Video Call, Social Media) and locations
- **Set reminders** to reach out to people on a regular cadence (including self-check-in reminders)
- **Organize** people into circles
- **Search** your network by name, nickname, or a part of a name
- **Archive** people and circles you're no longer actively keeping track of (without losing their data)

## Quick Start

```bash
# Run with sbt
sbt run

# Or build a JAR and run it
sbt assembly
java -jar target/scala-3.7.4/relationships.jar
```

Your data is stored at `~/.relationships/network.json` (or `%USERPROFILE%\.relationships\network.json` on Windows).

---

## Building from Source

### Prerequisites

- Java 17 or higher
- sbt 1.10+ (Scala Build Tool)

#### Installing sbt

**macOS:**
```bash
brew install sbt
```

**Linux:** See [Installing sbt on Linux](https://www.scala-sbt.org/1.x/docs/Installing-sbt-on-Linux.html)

**Windows:** See [Installing sbt on Windows](https://www.scala-sbt.org/1.x/docs/Installing-sbt-on-Windows.html)

### Running with sbt

```bash
cd relationships
sbt run
```

Or with `--help`:

```bash
sbt "run --help"
```

### Running Tests

```bash
sbt test
```

### Building a JAR

To build a standalone JAR file:

```bash
sbt assembly
```

This creates `target/scala-3.7.4/relationships.jar` which you can run with:

```bash
java -jar relationships.jar
```

---

## Usage

### First Run

On first run, you'll be prompted to enter your name. This creates your network file at `~/.relationships/network.json`.

```
Personal Relationship Manager
Type 'help' for commands, 'exit' to exit.

No existing network found.

What's your name? Petros
Welcome, Petros! Your network has been created.

>
```

### Commands

#### People

| Command | Description |
|---------|-------------|
| `list` | List all active people in your network |
| `add <name>` | Add a new person |
| `show <name>` | Show details about a person |
| `edit <name>` | Edit a person's information and labels |
| `search <query>` | Search for people by name |
| `archive <name>` | Archive a person (hide from main list) |
| `unarchive <name>` | Restore an archived person |
| `archived` | List archived people |

#### Interactions

| Command | Description |
|---------|-------------|
| `log <name>` | Log an interaction with someone |
| `remind` | Show people you're overdue to contact |
| `set-reminder <name>` | Set reminder frequency for someone |

When logging an interaction, you'll be prompted to select:
1. **Medium**: In Person, Text, Phone Call, Video Call, or Social Media
2. **Location(s)**: For in-person, one shared location. For remote, your location (required) and their location (optional).
3. **Topics**: What you discussed
4. **Note**: Optional additional notes

#### Contact Info

| Command | Description |
|---------|-------------|
| `add-phone <name>` | Add a phone number to someone |
| `add-email <name>` | Add an email address to someone |

#### Organization

| Command | Description |
|---------|-------------|
| `labels` | List all relationship labels |
| `circles` | List all active circles |
| `add-circle <name>` | Create a new circle (with option to add members) |
| `show-circle <name>` | Show circle details and members |
| `edit-circle <name>` | Edit circle name and members |
| `archive-circle <name>` | Archive a circle |
| `unarchive-circle <name>` | Restore an archived circle |
| `archived-circles` | List archived circles |

#### Other

| Command | Description |
|---------|-------------|
| `stats` | Show network statistics |
| `save` | Manually save (auto-saves after changes) |
| `help` | Show help |
| `exit` | Exit the program |

### Example Session

```
> add Alice
Added Alice
How did you meet? (press Enter to skip) Farmers market
Labels (enter numbers separated by spaces, or press Enter to skip):
  1. acquaintance
  2. coworker
  3. family
  4. friend
  ...
Labels: 4
Reminder every how many days? (press Enter to skip) 14
Reminder set for every 14 days

> log Alice
Logging interaction with Alice
How did you interact?
  1. In Person
  2. Text
  3. Phone Call
  4. Video Call
  5. Social Media
Medium (1-5): 1
Location: Coffee shop
Topics (comma-separated): farming, weather
Note (optional): Great chat about her greenhouse
Logged interaction with Alice

> edit Alice
Editing Alice (press Enter to keep current value)

Name [Alice]: 
Nickname []: Ali
How we met [Farmers market]: 
Notes []: Interested in permaculture
Default location []: Coffee shop

Labels (enter numbers to toggle, press Enter when done):
Current: friend

  1. [ ] acquaintance
  2. [ ] coworker
  3. [ ] family
  4. [x] friend
  ...
Toggle (or Enter to finish): 3
  1. [ ] acquaintance
  2. [ ] coworker
  3. [x] family
  4. [x] friend
  ...
Toggle (or Enter to finish): 

Updated Alice
Labels: family, friend

> remind
No overdue reminders! You're all caught up.
```

### Tips

- Names are matched case-insensitively
- Partial name matches work for most commands
- Data auto-saves after every change
- Use quotes for names with spaces: `add "Mary Jane"`

## Data Storage

Your network is stored in JSON format at `~/.relationships/network.json`.

To use a different file:

```bash
./relationships --file /path/to/mynetwork.json
```

### Backup

Your data is just a JSON file. Back it up however you normally back up files:

```bash
cp ~/.relationships/network.json ~/backups/
```

## Project Structure

```
relationships/
├── build.sbt                 # Build configuration
├── project/
│   ├── build.properties      # sbt version
│   └── plugins.sbt           # sbt plugins (native-image)
├── README.md                 # This file
└── src/
    ├── main/scala/network/
    │   ├── Model.scala       # Core data types
    │   ├── Validation.scala  # Input validation
    │   ├── NetworkOps.scala  # Operations (add, update, etc.)
    │   ├── NetworkQueries.scala  # Queries (search, filter, etc.)
    │   ├── JsonCodecs.scala  # JSON serialization
    │   └── CLI.scala         # Command-line interface
    └── test/scala/network/
        ├── ModelSpec.scala
        ├── NetworkOpsSpec.scala
        ├── NetworkQueriesSpec.scala
        └── JsonCodecsSpec.scala
```

## Architecture

The codebase follows functional programming principles:

- **Immutable data** - All types are immutable case classes
- **Pure functions** - Operations return new values instead of mutating
- **Explicit errors** - Operations return `Either[ValidationError, Result]`
- **Separation of concerns** - Model, operations, queries, and serialization are separate

### Key Types

- `InteractionMedium` - How you interacted (InPerson, Text, PhoneCall, VideoCall, SocialMedia)
- `Interaction` - Records of interactions with `medium`, `myLocation`, `theirLocation`, `topics`, `note`
- `Person` - People in your network with contact info, notes, and archiving
- `Circle` - Organizational groupings with archiving support
- `RelationshipLabel` - Labels describing your relationship to someone
