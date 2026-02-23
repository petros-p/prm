# Personal Relationship Manager (PRM)

A local-first CLI tool for tracking relationships, interactions, and reminders.
Privacy-first: all data stays on your machine. No external APIs.

## Setup

1. Install [Rust](https://rustup.rs/) (stable, 1.93+)
2. Install [Ollama](https://ollama.com/) for AI features, then pull the default model:
   ```bash
   ollama pull llama3.2:3b
   ```
3. Clone and run:
   ```bash
   git clone https://github.com/petros-p/prm.git
   cd prm
   cargo run
   ```

Data is stored in `.data/prm.db` (SQLite).

## Commands

### People
| Command | Description |
|---------|-------------|
| `people` | List all people |
| `add-person [name]` | Add a person (interactive) |
| `show-person <name>` | Show person details (with recent interactions) |
| `history <name>` | Show full interaction history |
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
| `log <name>` | Log an interaction (manual) |
| `ai-log <description>` | Log via AI (natural language, local Ollama) |
| `voice-log <wav-file>` | Log via voice recording (local Whisper transcription) |
| `remind` | Show overdue and upcoming reminders |
| `set-reminder <name>` | Set reminder frequency |

### Other
| Command | Description |
|---------|-------------|
| `stats` | Show statistics |
| `help` | Show all commands |
| `exit` / `quit` / `q` | Exit |

## AI Features

All AI runs locally — no API keys, no data leaves your machine.

- **`ai-log`** — Describe an interaction in plain text; Ollama parses it into structured data for review and save.
- **`voice-log`** — Record a `.wav` file; Whisper transcribes it locally, then Ollama parses it.

### Voice log setup

Download the Whisper model (~148MB) and place it at `.data/models/ggml-base.en.bin`:
```bash
mkdir -p .data/models
curl -L -o .data/models/ggml-base.en.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin
```

### Environment variables
| Variable | Default | Description |
|----------|---------|-------------|
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama server URL |
| `PRM_MODEL` | `llama3.2:3b` | Ollama model to use |
| `PRM_WHISPER_MODEL` | `.data/models/ggml-base.en.bin` | Whisper model path |

## Building

```bash
cargo build --release    # Optimized binary at target/release/prm
cargo test               # Run all tests (99 tests)
cargo check              # Fast compile check
```
