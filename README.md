# Personal Relationship Manager (PRM)

A CLI tool for tracking relationships, interactions, and reminders to stay in touch.
This is a working prototype for proof of concept.

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

Type `exit`, `quit`, or `q` to exit at any time.

## Commands

### People
| Command | Description |
|---------|-------------|
| `people` | List all people |
| `add-person [name]` | Add a person (interactive) |
| `show-person <name>` | Show person details |
| `edit-person <name>` | Edit a person (menu) |
| `find <query>` | Search people, circles, labels |
| `archive-person <name>` | Archive a person |
| `unarchive-person <name>` | Restore archived person |
| `archived-people` | List archived people |

### Quick Edits
| Command | Description |
|---------|-------------|
| `edit-name <name>` | Edit person's name |
| `edit-nickname <name>` | Edit nickname |
| `edit-birthday <name>` | Edit birthday |
| `edit-how-we-met <name>` | Edit how you met |
| `edit-notes <name>` | Edit notes |
| `edit-location <name>` | Edit location |
| `edit-labels <name>` | Edit labels |
| `edit-circles <name>` | Edit circles |
| `edit-phone <name>` | Edit phone numbers |
| `edit-email <name>` | Edit email addresses |

### Circles
| Command | Description |
|---------|-------------|
| `circles` | List all circles |
| `add-circle [name]` | Create a circle |
| `show-circle <name>` | Show circle details |
| `edit-circle <name>` | Edit a circle |
| `archive-circle <name>` | Archive a circle |
| `unarchive-circle <name>` | Restore archived circle |
| `archived-circles` | List archived circles |

### Labels
| Command | Description |
|---------|-------------|
| `labels` | List all labels |
| `add-label [name]` | Create a label |
| `show-label <name>` | Show label details |
| `edit-label <name>` | Edit a label |
| `archive-label <name>` | Archive a label |
| `unarchive-label <name>` | Restore archived label |
| `archived-labels` | List archived labels |

### Interactions & Reminders
| Command | Description |
|---------|-------------|
| `log <name>` | Log an interaction |
| `remind` | Show overdue reminders |
| `set-reminder <name>` | Set reminder frequency |

### Other
| Command | Description |
|---------|-------------|
| `stats` | Show statistics |
| `help` | Show all commands |
| `exit` / `quit` / `q` | Exit |

## Code Structure

```
src/main/scala/network/
├── Model.scala              # Data types (Person, Circle, Label, etc.)
├── NetworkOps.scala         # Operations (add, update, delete)
├── NetworkQueries.scala     # Queries (find, filter, stats)
├── JsonCodecs.scala         # JSON serialization
├── Validation.scala         # Input validation
├── CLI.scala                # Main entry point and REPL
├── PersonCommands.scala     # Person CLI commands
├── CircleCommands.scala     # Circle CLI commands
├── LabelCommands.scala      # Label CLI commands
└── InteractionCommands.scala # Interaction/reminder commands
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

- [ ] Automated interaction logging (email, calendar, texts)
- [ ] Relationship strength scoring
- [ ] Mobile app
- [ ] Important event reminders
- [ ] Data export
- [ ] Multi-device sync
- [ ] Web app
- [ ] Import contacts from phone/email

## Example Session

```
> add-person
Adding a new person (press Enter to skip optional fields, 's' to save and exit)

Name (required): Guy Testadopoulos
Added Guy Testadopoulos
Nickname: Guy
Birthday (YYYY-MM-DD): 1990-05-15
How did you meet: Work conference
Notes: Great conversation about hiking
Location: Boston

Add labels? (y/n): y
Select labels (enter numbers to toggle, Enter when done):
  1. [ ] acquaintance
  2. [ ] coworker
  3. [ ] family
  4. [ ] friend
Toggle (or Enter to finish): 2 4
Labels: coworker, friend

Add to circles? (y/n): n

Add phone numbers? (y/n): y
Phone number (or Enter to finish): 555-1234
Label (optional): work
Added: 555-1234
Phone number (or Enter to finish): 

Add email addresses? (y/n): n

Set reminder? (y/n): y
Remind every how many days: 14
Reminder set for every 14 days

Log an interaction now? (y/n): n

Finished adding Guy Testadopoulos.

> show-person Guy
Name: Guy Testadopoulos
Nickname: Guy
Birthday: 1990-05-15
How we met: Work conference
Notes: Great conversation about hiking
Location: Boston
Labels: coworker, friend
Circles: (none)
Phones: 555-1234 (work)
Emails: (none)
Reminder: every 14 days
Last interaction: (never)
Total interactions: 0
```
