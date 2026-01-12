# Personal Relationship Manager

A command-line tool for tracking your relationships. Keep notes on people you know, log interactions, and get reminders to stay in touch.

## Features

- **Track people** in your network with notes, labels, and contact info
- **Log interactions** with medium (In Person, Text, Phone Call, Video Call, Social Media) and locations
- **Set reminders** to reach out to people on a regular cadence
- **Organize** people into circles
- **Search** your network by name, nickname, or a part of a name
- **Archive** people and circles you're no longer actively keeping track of (without losing their data)

## Installation

### Prerequisites

- Java 17
- sbt 1.12.0 (Scala Build Tool)

If you don't have sbt, install it via:

#### macOS
See [Installing sbt on Mac](https://www.scala-sbt.org/1.x/docs/Installing-sbt-on-Mac.html)

#### Linux
See [Installing sbt on Linux](https://www.scala-sbt.org/1.x/docs/Installing-sbt-on-Linux.html)

#### Windows
See [Installing sbt on Windows](https://www.scala-sbt.org/1.x/docs/Installing-sbt-on-Windows.html)

### Running

```bash
cd relationships
sbt run
```

Or with `--help`:

```bash
sbt "run --help"
```

## Usage

### First Run

On first run, you'll be prompted to enter your name. This creates your network file at `~/.relationships/network.json`.

```
Personal Relationship Manager
Type 'help' for commands, 'exit' or 'quit' to exit.

No existing network found.

What's your name? Testy Testadopoulos
Welcome, Testy! Your network has been created.

>
```

### Commands

#### People

| Command | Description |
|---------|-------------|
| `list` | List all active people in your network |
| `add <n>` | Add a new person |
| `show <n>` | Show details about a person |
| `edit <n>` | Edit a person's information |
| `edit-labels <n>` | Add/remove labels for a person |
| `search <query>` | Search for people by name |
| `archive <n>` | Archive a person (hide from main list) |
| `unarchive <n>` | Restore an archived person |
| `archived` | List archived people |

#### Interactions

| Command | Description |
|---------|-------------|
| `log <n>` | Log an interaction with someone |
| `remind` | Show people you're overdue to contact |
| `set-reminder <n>` | Set reminder frequency for someone |

When logging an interaction, you'll be prompted to select:
1. **Medium**: In Person, Text, Phone Call, Video Call, or Social Media
2. **Location(s)**: For in-person, one shared location. For remote, your location (required) and their location (optional).
3. **Topics**: What you discussed
4. **Note**: Optional additional notes

#### Contact Info

| Command | Description |
|---------|-------------|
| `add-phone <n>` | Add a phone number to someone |
| `add-email <n>` | Add an email address to someone |

#### Organization

| Command | Description |
|---------|-------------|
| `labels` | List all relationship labels |
| `circles` | List all active circles |
| `add-circle <n>` | Create a new circle (with option to add members) |
| `show-circle <n>` | Show circle details and members |
| `edit-circle <n>` | Edit circle name and members |
| `archive-circle <n>` | Archive a circle |
| `unarchive-circle <n>` | Restore an archived circle |
| `archived-circles` | List archived circles |

#### Other

| Command | Description |
|---------|-------------|
| `stats` | Show network statistics |
| `save` | Manually save (auto-saves after changes) |
| `help` | Show help |
| `quit` | Exit the program |

### Examples

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
Location [Coffee shop]: 
Topics (comma-separated): farming, weather
Note (optional): Great chat about her greenhouse
Logged interaction with Alice

> add-circle "Farm Friends"
Circle name: Farm Friends
Add members (enter numbers separated by spaces, or press Enter to skip):
  1. Alice
  2. Bob
Members: 1 2
Created circle: Farm Friends with 2 members

> edit-labels Alice
Editing labels for Alice

Current labels: friend

Available labels (enter numbers to toggle, press Enter when done):
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
Labels updated: family, friend

> remind
People to reach out to (1):

  Bob - 15 days overdue (last contact 6 week(s) ago)

> show Alice
Name: Alice
How we met: Farmers market
Labels: family, friend
Circles: Farm Friends
Reminder: every 14 days
Last interaction: today via In Person
  My location: Coffee shop
  Topics: farming, weather
  Note: Great chat about her greenhouse
Total interactions: 1
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
sbt "run --file /path/to/mynetwork.json"
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

## Development

### Running Tests

```bash
sbt test
```

### Architecture

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
