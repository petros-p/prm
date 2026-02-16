pub mod context;
pub mod person_commands;
pub mod circle_commands;
pub mod label_commands;
pub mod interaction_commands;
pub mod ai_log_command;

use std::path::Path;
use rusqlite::Connection;

use crate::db::{schema, network_repo, person_repo, relationship_repo};
use crate::model::*;
use context::CLIContext;

/// Run the interactive REPL.
pub fn run(db_path: &Path) {
    println!("Personal Relationship Manager");
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

    let ctx = match load_or_init(conn) {
        Some(ctx) => ctx,
        None => return,
    };

    repl_loop(&ctx);
}

fn load_or_init(conn: Connection) -> Option<CLIContext> {
    // Check if there's an existing user
    match network_repo::find_first_user(&conn) {
        Ok(Some(user)) => {
            let self_id = network_repo::get_self_id(&conn, user.id).ok().flatten();
            match self_id {
                Some(sid) => {
                    if let Some(self_person) = person_repo::find_by_id(&conn, sid).ok().flatten() {
                        println!("Loaded network for {}", self_person.name);
                        return Some(CLIContext::new(conn, user, sid));
                    }
                    println!("Error: self person not found. Starting fresh...");
                }
                None => {
                    println!("Error: network metadata missing. Starting fresh...");
                }
            }
            // Fall through to init with existing user but missing metadata
            init_new_network(conn)
        }
        Ok(None) => {
            println!("No existing network found.");
            init_new_network(conn)
        }
        Err(e) => {
            println!("Error loading data: {}", e);
            println!("Starting fresh...");
            init_new_network(conn)
        }
    }
}

fn init_new_network(conn: Connection) -> Option<CLIContext> {
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

    Some(CLIContext::new(conn, user, self_person.id))
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
            "ai-log" => ai_log_command::ai_log(ctx, args),

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

fn print_help() {
    println!(r#"
COMMANDS:

  People:
    people                  List all people
    add-person [name]       Add a new person (interactive)
    show-person <name>      Show person details
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
    ai-log <description>    Log via AI (natural language)
    remind                  Show overdue reminders
    set-reminder <name>     Set reminder frequency

  Other:
    stats                   Show statistics
    help                    Show this help
    exit / quit / q         Exit

TIPS:
  - Names are case-insensitive and partial matches work
  - Press 's' during add-person to save and exit early"#);
}
