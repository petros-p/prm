pub mod context;
pub mod person_commands;
pub mod circle_commands;
pub mod label_commands;
pub mod interaction_commands;
#[cfg(feature = "ai")]
pub mod ai_log_command;
#[cfg(feature = "ai")]
pub mod voice_log_command;
#[cfg(feature = "ai")]
pub mod inbox_command;

use std::path::{Path, PathBuf};
use rusqlite::Connection;

use crate::db::{schema, network_repo, person_repo, relationship_repo};
use crate::model::*;
use crate::queries::reminder_queries;
use context::CLIContext;

/// Run the interactive REPL.
pub fn run(db_path: &Path) {
    println!("Personal Relationship Manager");
    println!("Keep track of the people in your life — log what you talked about, and get nudged who to reach out to next.");
    println!("Type 'help' for commands, 'exit' to quit.");
    println!();

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            return;
        }
    };

    if let Err(e) = schema::initialize(&conn) {
        eprintln!("Error initializing database: {}", e);
        return;
    }

    let inbox_dir = resolve_inbox_dir(db_path);
    #[cfg(feature = "ai")]
    if let Err(e) = std::fs::create_dir_all(&inbox_dir) {
        eprintln!("Warning: could not create inbox directory {}: {}", inbox_dir.display(), e);
    }

    let ctx = match load_or_init(conn, inbox_dir) {
        Some(ctx) => ctx,
        None => return,
    };

    show_startup_reminders(&ctx);
    #[cfg(feature = "ai")]
    show_inbox_hint(&ctx);

    repl_loop(&ctx);
}

/// Resolve the watched inbox folder: `PRM_INBOX` env var, or `<db_dir>/inbox`.
fn resolve_inbox_dir(db_path: &Path) -> PathBuf {
    if let Ok(p) = std::env::var("PRM_INBOX") {
        return PathBuf::from(p);
    }
    db_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("inbox")
}

#[cfg(feature = "ai")]
fn show_inbox_hint(ctx: &CLIContext) {
    let count = inbox_command::pending_count(&ctx.inbox_dir);
    if count > 0 {
        println!(
            "{} new note{} captured — run 'inbox' to process {}.",
            count,
            if count == 1 { "" } else { "s" },
            if count == 1 { "it" } else { "them" }
        );
        println!();
    }
}

fn load_or_init(conn: Connection, inbox_dir: PathBuf) -> Option<CLIContext> {
    // Check if there's an existing user
    match network_repo::find_first_user(&conn) {
        Ok(Some(user)) => {
            let self_id = network_repo::get_self_id(&conn, user.id).ok().flatten();
            match self_id {
                Some(sid) => {
                    if let Some(self_person) = person_repo::find_by_id(&conn, sid).ok().flatten() {
                        println!("Loaded network for {}", self_person.name);
                        return Some(CLIContext::new(conn, user, sid, inbox_dir));
                    }
                    println!("Error: self person not found. Starting fresh...");
                }
                None => {
                    println!("Error: network metadata missing. Starting fresh...");
                }
            }
            // Fall through to init with existing user but missing metadata
            init_new_network(conn, inbox_dir)
        }
        Ok(None) => {
            println!("No existing network found.");
            init_new_network(conn, inbox_dir)
        }
        Err(e) => {
            println!("Error loading data: {}", e);
            println!("Starting fresh...");
            init_new_network(conn, inbox_dir)
        }
    }
}

fn init_new_network(conn: Connection, inbox_dir: PathBuf) -> Option<CLIContext> {
    println!();
    print!("What's your name? ");
    use std::io::Write;
    std::io::stdout().flush().ok();

    let mut name = String::new();
    std::io::stdin().read_line(&mut name).ok()?;
    let name = name.trim().to_string();

    let name_lower = name.to_lowercase();
    if name_lower == "exit" || name_lower == "quit" || name_lower == "q" {
        return None;
    }

    if name.is_empty() {
        println!("Name cannot be empty. Please restart and try again.");
        return None;
    }

    let user = User::create(name.clone(), String::new());
    network_repo::insert_user(&conn, &user).ok()?;

    let self_person = Person::create_self(name.clone());
    person_repo::insert(&conn, user.id, &self_person).ok()?;
    network_repo::set_network_metadata(&conn, user.id, self_person.id).ok()?;

    // Create default labels
    let defaults = RelationshipLabel::defaults();
    for label in &defaults {
        let _ = relationship_repo::insert_label(&conn, user.id, label);
    }

    // Create relationship for self and assign "me" label
    let me_label = defaults.iter().find(|l| l.name == "me");
    if let Some(me) = me_label {
        let rel = Relationship {
            person_id: self_person.id,
            labels: vec![me.id],
            reminder_days: None,
        };
        let _ = relationship_repo::upsert(&conn, user.id, &rel);
    }

    println!("Welcome, {}! Your network has been created.", name);
    println!();
    println!("Next steps:");
    println!("  add-person <name>       Add your first contact");
    println!("  log <name>              Log an interaction by hand");
    #[cfg(feature = "ai")]
    println!("  ai-log <description>    Or just describe it in plain text");
    #[cfg(feature = "ai")]
    println!("  inbox                   Process notes dropped in {}", inbox_dir.display());
    println!("  remind                  See who's due for a check-in");
    println!();

    Some(CLIContext::new(conn, user, self_person.id, inbox_dir))
}

fn show_startup_reminders(ctx: &CLIContext) {
    const DUE_SOON_DAYS: i64 = 7;
    let today = CLIContext::today();
    // Already sorted most-urgent-first (never contacted, then most overdue,
    // down through soonest-due) by reminder_queries::all_reminders, so this
    // filter preserves that order rather than needing its own sort.
    let all = reminder_queries::all_reminders(&ctx.conn, ctx.owner_id(), today)
        .unwrap_or_default();

    let due: Vec<_> = all.iter().filter(|s| match &s.overdue_status {
        reminder_queries::OverdueStatus::NeverContacted => true,
        reminder_queries::OverdueStatus::DaysOverdue(d) => *d > -DUE_SOON_DAYS,
    }).collect();

    if due.is_empty() {
        return;
    }

    println!("Reach out to next:");
    for status in &due {
        let detail = match &status.overdue_status {
            reminder_queries::OverdueStatus::NeverContacted => "never contacted".to_string(),
            reminder_queries::OverdueStatus::DaysOverdue(d) if *d > 0 => {
                format!("{} day{} overdue", d, if *d == 1 { "" } else { "s" })
            }
            reminder_queries::OverdueStatus::DaysOverdue(0) => "due today".to_string(),
            reminder_queries::OverdueStatus::DaysOverdue(d) => {
                format!("due in {} day{}", -d, if *d == -1 { "" } else { "s" })
            }
        };
        println!("  {} — {}", status.person.name, detail);
    }
    println!();
}

fn repl_loop(ctx: &CLIContext) {
    loop {
        let input = match ctx.read_line("> ") {
            Some(s) => s,
            None => break,
        };

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let (command, args) = parse_command(input);

        match command {
            "help" | "?" => print_help(),
            "quit" | "exit" | "q" => break,

            // Person commands
            "people" | "list" | "ls" => person_commands::list(ctx),
            "add-person" => person_commands::add(ctx, args),
            "show-person" | "show" | "view" => person_commands::show(ctx, args),
            "history" => person_commands::history(ctx, args),
            "edit-person" => person_commands::edit(ctx, args),
            "find" => person_commands::find(ctx, args),
            "archive-person" => person_commands::archive(ctx, args),
            "unarchive-person" => person_commands::unarchive(ctx, args),
            "archived-people" => person_commands::list_archived(ctx),

            // Granular person edits
            "edit-name" => person_commands::edit_name(ctx, args),
            "edit-nickname" => person_commands::edit_nickname(ctx, args),
            "edit-birthday" => person_commands::edit_birthday(ctx, args),
            "edit-how-we-met" => person_commands::edit_how_we_met(ctx, args),
            "edit-notes" => person_commands::edit_notes(ctx, args),
            "edit-location" => person_commands::edit_location(ctx, args),
            "edit-labels" => person_commands::edit_labels(ctx, args),
            "edit-circles" => person_commands::edit_circles(ctx, args),
            "edit-phone" => person_commands::edit_phone(ctx, args),
            "edit-email" => person_commands::edit_email(ctx, args),

            // Circle commands
            "circles" => circle_commands::list(ctx),
            "add-circle" => circle_commands::add(ctx, args),
            "show-circle" => circle_commands::show(ctx, args),
            "edit-circle" => circle_commands::edit(ctx, args),
            "archive-circle" => circle_commands::archive(ctx, args),
            "unarchive-circle" => circle_commands::unarchive(ctx, args),
            "archived-circles" => circle_commands::list_archived(ctx),

            // Label commands
            "labels" => label_commands::list(ctx),
            "add-label" => label_commands::add(ctx, args),
            "show-label" => label_commands::show(ctx, args),
            "edit-label" => label_commands::edit(ctx, args),
            "archive-label" => label_commands::archive(ctx, args),
            "unarchive-label" => label_commands::unarchive(ctx, args),
            "archived-labels" => label_commands::list_archived(ctx),

            // Interaction commands
            "log" => interaction_commands::log(ctx, args),
            "remind" | "reminders" => interaction_commands::show_reminders(ctx),
            "set-reminder" => interaction_commands::set_reminder(ctx, args),

            // AI-assisted
            #[cfg(feature = "ai")]
            "ai-log" => ai_log_command::ai_log(ctx, args),
            #[cfg(feature = "ai")]
            "voice-log" => voice_log_command::voice_log(ctx, args),
            #[cfg(feature = "ai")]
            "inbox" => inbox_command::inbox(ctx, args),

            // Other
            "stats" => interaction_commands::print_stats(ctx),

            _ => println!("Unknown command: {}. Type 'help' for commands.", command),
        }
    }
}

/// Parse input into command and args, handling quoted strings.
fn parse_command(input: &str) -> (&str, &str) {
    let input = input.trim();
    match input.find(|c: char| c == ' ' || c == '\t') {
        Some(pos) => (&input[..pos], input[pos..].trim()),
        None => (input, ""),
    }
}

#[cfg(feature = "ai")]
fn ai_help_lines() -> &'static str {
    "    ai-log <description>    Log via AI (natural language)\n    voice-log <wav-file>    Log via voice recording (local Whisper transcription)\n    inbox                   Process notes dropped in your watched inbox folder\n"
}

#[cfg(not(feature = "ai"))]
fn ai_help_lines() -> &'static str {
    ""
}

#[cfg(feature = "ai")]
fn getting_started_log_line() -> &'static str {
    "  2. Log an interaction — 'log <name>' by hand, 'ai-log <description>' in plain\n     text, or drop a note in your inbox folder and run 'inbox'\n"
}

#[cfg(not(feature = "ai"))]
fn getting_started_log_line() -> &'static str {
    "  2. Log an interaction — 'log <name>'\n"
}

fn print_help() {
    println!(
        r#"
PRM helps you keep in touch with people: track who's in your life, log what
you talk about, and get nudged when it's time to reach out again.

GETTING STARTED:
  1. Add someone — 'add-person <name>'
{}  3. Check in — 'remind' shows who's due for a check-in

COMMANDS:

  People:
    people                  List all people
    add-person [name]       Add a new person (interactive)
    show-person <name>      Show person details
    history <name>          Show full interaction history
    edit-person <name>      Edit a person (menu)
    find <query>            Search people, circles, and labels
    archive-person <name>   Archive a person
    unarchive-person <name> Restore archived person
    archived-people         List archived people

  Person Quick Edits:
    edit-name <name>        Edit person's name
    edit-nickname <name>    Edit person's nickname
    edit-birthday <name>    Edit person's birthday
    edit-how-we-met <name>  Edit how you met
    edit-notes <name>       Edit person's notes
    edit-location <name>    Edit person's location
    edit-labels <name>      Edit person's labels
    edit-circles <name>     Edit person's circles
    edit-phone <name>       Edit person's phone numbers
    edit-email <name>       Edit person's email addresses

  Circles:
    circles                 List all circles
    add-circle [name]       Create a new circle
    show-circle <name>      Show circle details
    edit-circle <name>      Edit a circle
    archive-circle <name>   Archive a circle
    unarchive-circle <name> Restore archived circle
    archived-circles        List archived circles

  Labels:
    labels                  List all labels
    add-label [name]        Create a new label
    show-label <name>       Show label details
    edit-label <name>       Edit a label
    archive-label <name>    Archive a label
    unarchive-label <name>  Restore archived label
    archived-labels         List archived labels

  Interactions:
    log <name>              Log an interaction (manual prompts)
{}    remind                  Show overdue reminders
    set-reminder <name>     Set reminder frequency

  Other:
    stats                   Show statistics
    help                    Show this help
    exit / quit / q         Exit

TIPS:
  - Names are case-insensitive and partial matches work
  - Press 's' during add-person to save and exit early"#,
        getting_started_log_line(),
        ai_help_lines()
    );
}
