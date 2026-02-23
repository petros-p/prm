use chrono::NaiveDate;

use crate::cli::context::CLIContext;
use crate::model::*;
use crate::ops::*;
use crate::queries::*;

pub fn list(ctx: &CLIContext) {
    let people = person_queries::active_people(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    if people.is_empty() {
        println!("No people in your network yet. Use 'add-person' to add someone.");
        return;
    }

    println!("People in your network ({}):", people.len());
    println!();
    for person in &people {
        let labels = ctx.labels_for(person.id);
        let label_str = if labels.is_empty() {
            String::new()
        } else {
            let mut names: Vec<&str> = labels.iter().map(|l| l.name.as_str()).collect();
            names.sort();
            format!(" [{}]", names.join(", "))
        };

        let last_contact = interaction_queries::days_since_interaction(
            &ctx.conn,
            person.id,
            CLIContext::today(),
        )
        .ok()
        .flatten()
        .map(|d| format!(" - last contact: {}", CLIContext::format_days_ago(d)))
        .unwrap_or_default();

        println!("  {}{}{}", person.name, label_str, last_contact);
    }
}

pub fn add(ctx: &CLIContext, args: &str) {
    println!("Adding a new person (press Enter to skip optional fields, 's' to save and exit)");
    println!();

    let name = if !args.is_empty() {
        args.to_string()
    } else {
        match ctx.prompt("Name (required): ") {
            Some(s) if s.eq_ignore_ascii_case("s") || s.is_empty() => {
                if s.is_empty() {
                    println!("Name is required.");
                }
                return;
            }
            Some(s) => s,
            None => return,
        }
    };

    let person = match person_ops::add_person(&ctx.conn, ctx.owner_id(), &name, None, None, None, None, None) {
        Ok(p) => {
            println!("Added {}", p.name);
            p
        }
        Err(e) => {
            ctx.print_error(&e);
            return;
        }
    };

    macro_rules! prompt_or_save {
        ($prompt:expr) => {
            match ctx.prompt($prompt) {
                Some(s) if s.eq_ignore_ascii_case("s") => {
                    println!("Saved.");
                    return;
                }
                Some(s) => s,
                None => return,
            }
        };
    }

    // Nickname
    let nickname = prompt_or_save!("Nickname: ");
    if !nickname.is_empty() {
        let _ = person_ops::update_person(&ctx.conn, person.id, None, Some(Some(&nickname)), None, None, None, None);
    }

    // Birthday
    let birthday = prompt_or_save!("Birthday (YYYY-MM-DD): ");
    if !birthday.is_empty() {
        match NaiveDate::parse_from_str(&birthday, "%Y-%m-%d") {
            Ok(date) => {
                let _ = person_ops::update_person(&ctx.conn, person.id, None, None, None, Some(Some(date)), None, None);
            }
            Err(_) => println!("Invalid date format, skipping."),
        }
    }

    // How we met
    let how_we_met = prompt_or_save!("How did you meet: ");
    if !how_we_met.is_empty() {
        let _ = person_ops::update_person(&ctx.conn, person.id, None, None, Some(Some(&how_we_met)), None, None, None);
    }

    // Notes
    let notes = prompt_or_save!("Notes: ");
    if !notes.is_empty() {
        let _ = person_ops::update_person(&ctx.conn, person.id, None, None, None, None, Some(Some(&notes)), None);
    }

    // Location
    let location = prompt_or_save!("Location: ");
    if !location.is_empty() {
        let _ = person_ops::update_person(&ctx.conn, person.id, None, None, None, None, None, Some(Some(&location)));
    }

    // Labels
    println!();
    let add_labels = prompt_or_save!("Add labels? (y/n): ");
    if add_labels.eq_ignore_ascii_case("y") {
        select_labels(ctx, person.id);
    }

    // Circles
    println!();
    let add_circles = prompt_or_save!("Add to circles? (y/n): ");
    if add_circles.eq_ignore_ascii_case("y") {
        select_circles(ctx, person.id);
    }

    // Phones
    println!();
    let add_phones = prompt_or_save!("Add phone numbers? (y/n): ");
    if add_phones.eq_ignore_ascii_case("y") {
        add_phones_loop(ctx, person.id);
    }

    // Emails
    println!();
    let add_emails = prompt_or_save!("Add email addresses? (y/n): ");
    if add_emails.eq_ignore_ascii_case("y") {
        add_emails_loop(ctx, person.id);
    }

    // Reminder
    println!();
    let set_rem = prompt_or_save!("Set reminder? (y/n): ");
    if set_rem.eq_ignore_ascii_case("y") {
        set_reminder_for(ctx, person.id);
    }

    // Log interaction
    println!();
    let log_int = prompt_or_save!("Log an interaction now? (y/n): ");
    if log_int.eq_ignore_ascii_case("y") {
        if let Some(p) = person_queries::get_person(&ctx.conn, person.id).ok().flatten() {
            super::interaction_commands::log_for_person(ctx, &p);
        }
    }

    println!();
    println!("Finished adding {}.", name);
}

pub fn show(ctx: &CLIContext, args: &str) {
    let person = match if args.is_empty() {
        println!("Usage: show-person <name>");
        return;
    } else {
        ctx.find_person(args)
    } {
        Some(p) => p,
        None => return,
    };

    println!();
    println!("Name: {}", person.name);
    println!("Nickname: {}", person.nickname.as_deref().unwrap_or("(none)"));
    println!("Birthday: {}", person.birthday.map(|d| d.to_string()).unwrap_or_else(|| "(none)".into()));
    println!("How we met: {}", person.how_we_met.as_deref().unwrap_or("(none)"));
    println!("Notes: {}", person.notes.as_deref().unwrap_or("(none)"));
    println!("Location: {}", person.location.as_deref().unwrap_or("(none)"));

    let labels = ctx.labels_for(person.id);
    if labels.is_empty() {
        println!("Labels: (none)");
    } else {
        let mut names: Vec<&str> = labels.iter().map(|l| l.name.as_str()).collect();
        names.sort();
        println!("Labels: {}", names.join(", "));
    }

    let circles = ctx.circles_for(person.id);
    if circles.is_empty() {
        println!("Circles: (none)");
    } else {
        let names: Vec<&str> = circles.iter().map(|c| c.name.as_str()).collect();
        println!("Circles: {}", names.join(", "));
    }

    let contacts = ctx.contacts_for(person.id);
    println!("Phones: {}", format_contacts(&contacts, "Phone"));
    println!("Emails: {}", format_contacts(&contacts, "Email"));

    let rel = relationship_queries::get_relationship(&ctx.conn, person.id).ok().flatten();
    let reminder = rel.and_then(|r| r.reminder_days);
    println!("Reminder: {}", reminder.map(|d| format!("every {} days", d)).unwrap_or_else(|| "(none)".into()));

    let last = interaction_queries::last_interaction_with(&ctx.conn, person.id).ok().flatten();
    match last {
        Some(interaction) => {
            let days = interaction_queries::days_since_interaction(&ctx.conn, person.id, CLIContext::today())
                .ok()
                .flatten()
                .unwrap_or(0);
            println!("Last interaction: {} via {}", CLIContext::format_days_ago(days), interaction.medium.display_name());
        }
        None => println!("Last interaction: (never)"),
    }

    let interactions = interaction_queries::interactions_with(&ctx.conn, person.id).unwrap_or_default();
    println!("Total interactions: {}", interactions.len());
    println!();
}

pub fn edit(ctx: &CLIContext, args: &str) {
    let person = match if args.is_empty() {
        println!("Usage: edit-person <name>");
        return;
    } else {
        ctx.find_person(args)
    } {
        Some(p) => p,
        None => return,
    };

    println!("Editing {}", person.name);
    println!();
    println!("What would you like to edit?");
    println!("  1. Name");
    println!("  2. Nickname");
    println!("  3. Birthday");
    println!("  4. How we met");
    println!("  5. Notes");
    println!("  6. Location");
    println!("  7. Labels");
    println!("  8. Circles");
    println!("  9. Phone numbers");
    println!(" 10. Email addresses");
    println!();

    match ctx.prompt("Choice (1-10, or Enter to cancel): ").as_deref() {
        Some("1") => edit_name_cmd(ctx, &person),
        Some("2") => edit_nickname_cmd(ctx, &person),
        Some("3") => edit_birthday_cmd(ctx, &person),
        Some("4") => edit_how_we_met_cmd(ctx, &person),
        Some("5") => edit_notes_cmd(ctx, &person),
        Some("6") => edit_location_cmd(ctx, &person),
        Some("7") => select_labels(ctx, person.id),
        Some("8") => select_circles(ctx, person.id),
        Some("9") => edit_phones_cmd(ctx, &person),
        Some("10") => edit_emails_cmd(ctx, &person),
        Some("") => println!("Cancelled."),
        _ => println!("Invalid choice."),
    }
}

pub fn find(ctx: &CLIContext, args: &str) {
    if args.is_empty() {
        println!("Usage: find <query>");
        return;
    }

    let lower = args.to_lowercase();
    let people = person_queries::find_by_name(&ctx.conn, ctx.owner_id(), args).unwrap_or_default();
    let all_circles = circle_queries::active_circles(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    let circles: Vec<&Circle> = all_circles.iter().filter(|c| c.name.to_lowercase().contains(&lower)).collect();
    let all_labels = relationship_queries::active_labels(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    let labels: Vec<&RelationshipLabel> = all_labels.iter().filter(|l| l.name.to_lowercase().contains(&lower)).collect();

    let total = people.len() + circles.len() + labels.len();
    if total == 0 {
        println!("No results found for '{}'", args);
        return;
    }

    println!("Found {} result(s) for '{}':", total, args);
    println!();

    if !people.is_empty() {
        println!("People:");
        for p in &people {
            let archived = if p.archived { " (archived)" } else { "" };
            let is_self = if p.is_self { " (you)" } else { "" };
            println!("  {}{}{}", p.name, is_self, archived);
        }
        println!();
    }

    if !circles.is_empty() {
        println!("Circles:");
        for c in &circles {
            let archived = if c.archived { " (archived)" } else { "" };
            println!("  {}{}", c.name, archived);
        }
        println!();
    }

    if !labels.is_empty() {
        println!("Labels:");
        for l in &labels {
            let archived = if l.archived { " (archived)" } else { "" };
            println!("  {}{}", l.name, archived);
        }
    }
}

pub fn archive(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: archive-person <name>"); return; } else { ctx.find_person(args) } {
        Some(person) => match person_ops::archive_person(&ctx.conn, person.id) {
            Ok(_) => println!("Archived {}", person.name),
            Err(e) => ctx.print_error(&e),
        },
        None => {}
    }
}

pub fn unarchive(ctx: &CLIContext, args: &str) {
    if args.is_empty() {
        println!("Usage: unarchive-person <name>");
        return;
    }

    let archived = person_queries::archived_people(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    let lower = args.to_lowercase();
    let matches: Vec<&Person> = archived.iter().filter(|p| {
        p.name.to_lowercase().contains(&lower)
            || p.nickname.as_ref().map(|n| n.to_lowercase().contains(&lower)).unwrap_or(false)
    }).collect();

    match matches.len() {
        0 => println!("No archived person found matching '{}'", args),
        1 => match person_ops::unarchive_person(&ctx.conn, matches[0].id) {
            Ok(_) => println!("Restored {}", matches[0].name),
            Err(e) => ctx.print_error(&e),
        },
        _ => {
            println!("Multiple matches found:");
            for p in &matches { println!("  {}", p.name); }
            println!("Please be more specific.");
        }
    }
}

pub fn list_archived(ctx: &CLIContext) {
    let archived = person_queries::archived_people(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    if archived.is_empty() {
        println!("No archived people.");
    } else {
        println!("Archived people ({}):", archived.len());
        for p in &archived { println!("  {}", p.name); }
    }
}

// Granular edit commands
pub fn edit_name(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-name <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => edit_name_cmd(ctx, &p),
        None => {}
    }
}

pub fn edit_nickname(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-nickname <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => edit_nickname_cmd(ctx, &p),
        None => {}
    }
}

pub fn edit_birthday(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-birthday <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => edit_birthday_cmd(ctx, &p),
        None => {}
    }
}

pub fn edit_how_we_met(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-how-we-met <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => edit_how_we_met_cmd(ctx, &p),
        None => {}
    }
}

pub fn edit_notes(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-notes <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => edit_notes_cmd(ctx, &p),
        None => {}
    }
}

pub fn edit_location(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-location <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => edit_location_cmd(ctx, &p),
        None => {}
    }
}

pub fn edit_labels(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-labels <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => select_labels(ctx, p.id),
        None => {}
    }
}

pub fn edit_circles(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-circles <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => select_circles(ctx, p.id),
        None => {}
    }
}

pub fn edit_phone(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-phone <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => edit_phones_cmd(ctx, &p),
        None => {}
    }
}

pub fn edit_email(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: edit-email <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => edit_emails_cmd(ctx, &p),
        None => {}
    }
}

// Internal edit implementations

fn edit_name_cmd(ctx: &CLIContext, person: &Person) {
    let input = match ctx.prompt(&format!("Name [{}]: ", person.name)) {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };
    match person_ops::update_person(&ctx.conn, person.id, Some(&input), None, None, None, None, None) {
        Ok(_) => println!("Updated name to: {}", input),
        Err(e) => ctx.print_error(&e),
    }
}

fn edit_nickname_cmd(ctx: &CLIContext, person: &Person) {
    let current = person.nickname.as_deref().unwrap_or("");
    let input = match ctx.prompt(&format!("Nickname [{}] (enter 'clear' to remove): ", current)) {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };
    let new_val = if input.eq_ignore_ascii_case("clear") { None } else { Some(input.as_str()) };
    match person_ops::update_person(&ctx.conn, person.id, None, Some(new_val), None, None, None, None) {
        Ok(_) => match new_val {
            Some(v) => println!("Updated nickname to: {}", v),
            None => println!("Cleared nickname."),
        },
        Err(e) => ctx.print_error(&e),
    }
}

fn edit_birthday_cmd(ctx: &CLIContext, person: &Person) {
    let current = person.birthday.map(|d| d.to_string()).unwrap_or_default();
    let input = match ctx.prompt(&format!("Birthday [{}] (YYYY-MM-DD, 'clear' to remove): ", current)) {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };
    if input.eq_ignore_ascii_case("clear") {
        match person_ops::update_person(&ctx.conn, person.id, None, None, None, Some(None), None, None) {
            Ok(_) => println!("Cleared birthday."),
            Err(e) => ctx.print_error(&e),
        }
    } else {
        match NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
            Ok(date) => match person_ops::update_person(&ctx.conn, person.id, None, None, None, Some(Some(date)), None, None) {
                Ok(_) => println!("Updated birthday to: {}", date),
                Err(e) => ctx.print_error(&e),
            },
            Err(_) => println!("Invalid date format."),
        }
    }
}

fn edit_how_we_met_cmd(ctx: &CLIContext, person: &Person) {
    let current = person.how_we_met.as_deref().unwrap_or("");
    let input = match ctx.prompt(&format!("How we met [{}] ('clear' to remove): ", current)) {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };
    let new_val = if input.eq_ignore_ascii_case("clear") { None } else { Some(input.as_str()) };
    match person_ops::update_person(&ctx.conn, person.id, None, None, Some(new_val), None, None, None) {
        Ok(_) => match new_val {
            Some(v) => println!("Updated: {}", v),
            None => println!("Cleared."),
        },
        Err(e) => ctx.print_error(&e),
    }
}

fn edit_notes_cmd(ctx: &CLIContext, person: &Person) {
    let current = person.notes.as_deref().unwrap_or("");
    let input = match ctx.prompt(&format!("Notes [{}] ('clear' to remove): ", current)) {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };
    let new_val = if input.eq_ignore_ascii_case("clear") { None } else { Some(input.as_str()) };
    match person_ops::update_person(&ctx.conn, person.id, None, None, None, None, Some(new_val), None) {
        Ok(_) => match new_val {
            Some(_) => println!("Updated notes."),
            None => println!("Cleared notes."),
        },
        Err(e) => ctx.print_error(&e),
    }
}

fn edit_location_cmd(ctx: &CLIContext, person: &Person) {
    let current = person.location.as_deref().unwrap_or("");
    let input = match ctx.prompt(&format!("Location [{}] ('clear' to remove): ", current)) {
        Some(s) if !s.is_empty() => s,
        _ => return,
    };
    let new_val = if input.eq_ignore_ascii_case("clear") { None } else { Some(input.as_str()) };
    match person_ops::update_person(&ctx.conn, person.id, None, None, None, None, None, Some(new_val)) {
        Ok(_) => match new_val {
            Some(v) => println!("Updated location to: {}", v),
            None => println!("Cleared location."),
        },
        Err(e) => ctx.print_error(&e),
    }
}

fn edit_phones_cmd(ctx: &CLIContext, person: &Person) {
    let contacts = ctx.contacts_for(person.id);
    let phones: Vec<&ContactEntry> = contacts.iter().filter(|c| c.contact_type == ContactType::Phone).collect();

    println!();
    println!("Current phones:");
    if phones.is_empty() {
        println!("  (none)");
    } else {
        for (i, entry) in phones.iter().enumerate() {
            let label = entry.label.as_ref().map(|l| format!(" ({})", l)).unwrap_or_default();
            let value = match &entry.value { ContactValue::StringValue { value } => value.as_str(), _ => "" };
            println!("  {}. {}{}", i + 1, value, label);
        }
    }
    println!();
    println!("Options: 'add' to add, number to remove, Enter to finish");

    loop {
        let input = match ctx.prompt("Action: ") {
            Some(s) => s.to_lowercase(),
            None => break,
        };
        match input.as_str() {
            "" => break,
            "add" => {
                if let Some(number) = ctx.prompt("Phone number: ") {
                    if !number.is_empty() {
                        let label = ctx.prompt("Label (optional): ").unwrap_or_default();
                        let label_opt = if label.is_empty() { None } else { Some(label.as_str()) };
                        match contact_ops::add_phone(&ctx.conn, person.id, &number, label_opt) {
                            Ok(_) => println!("Added: {}", number),
                            Err(e) => ctx.print_error(&e),
                        }
                    }
                }
            }
            n if n.chars().all(|c| c.is_ascii_digit()) => {
                let idx: usize = n.parse().unwrap_or(0);
                if idx > 0 && idx <= phones.len() {
                    match contact_ops::remove_contact(&ctx.conn, phones[idx - 1].id) {
                        Ok(_) => println!("Removed."),
                        Err(e) => ctx.print_error(&e),
                    }
                } else {
                    println!("Invalid number.");
                }
            }
            _ => println!("Invalid input."),
        }
    }
}

fn edit_emails_cmd(ctx: &CLIContext, person: &Person) {
    let contacts = ctx.contacts_for(person.id);
    let emails: Vec<&ContactEntry> = contacts.iter().filter(|c| c.contact_type == ContactType::Email).collect();

    println!();
    println!("Current emails:");
    if emails.is_empty() {
        println!("  (none)");
    } else {
        for (i, entry) in emails.iter().enumerate() {
            let label = entry.label.as_ref().map(|l| format!(" ({})", l)).unwrap_or_default();
            let value = match &entry.value { ContactValue::StringValue { value } => value.as_str(), _ => "" };
            println!("  {}. {}{}", i + 1, value, label);
        }
    }
    println!();
    println!("Options: 'add' to add, number to remove, Enter to finish");

    loop {
        let input = match ctx.prompt("Action: ") {
            Some(s) => s.to_lowercase(),
            None => break,
        };
        match input.as_str() {
            "" => break,
            "add" => {
                if let Some(email) = ctx.prompt("Email address: ") {
                    if !email.is_empty() {
                        let label = ctx.prompt("Label (optional): ").unwrap_or_default();
                        let label_opt = if label.is_empty() { None } else { Some(label.as_str()) };
                        match contact_ops::add_email(&ctx.conn, person.id, &email, label_opt) {
                            Ok(_) => println!("Added: {}", email),
                            Err(e) => ctx.print_error(&e),
                        }
                    }
                }
            }
            n if n.chars().all(|c| c.is_ascii_digit()) => {
                let idx: usize = n.parse().unwrap_or(0);
                if idx > 0 && idx <= emails.len() {
                    match contact_ops::remove_contact(&ctx.conn, emails[idx - 1].id) {
                        Ok(_) => println!("Removed."),
                        Err(e) => ctx.print_error(&e),
                    }
                } else {
                    println!("Invalid number.");
                }
            }
            _ => println!("Invalid input."),
        }
    }
}

fn select_labels(ctx: &CLIContext, person_id: Id<Person>) {
    let all_labels = relationship_queries::active_labels(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    if all_labels.is_empty() {
        println!("No labels available.");
        return;
    }

    let current = ctx.labels_for(person_id);
    let mut selected: Vec<Id<RelationshipLabel>> = current.iter().map(|l| l.id).collect();

    println!("Select labels (enter numbers to toggle, Enter when done):");

    loop {
        for (i, label) in all_labels.iter().enumerate() {
            let marker = if selected.contains(&label.id) { "[x]" } else { "[ ]" };
            println!("  {}. {} {}", i + 1, marker, label.name);
        }

        let input = match ctx.prompt("Toggle (or Enter to finish): ") {
            Some(s) if s.is_empty() => break,
            Some(s) => s,
            None => break,
        };

        for token in input.split_whitespace() {
            if let Ok(idx) = token.parse::<usize>() {
                if idx > 0 && idx <= all_labels.len() {
                    let label_id = all_labels[idx - 1].id;
                    if let Some(pos) = selected.iter().position(|id| *id == label_id) {
                        selected.remove(pos);
                    } else {
                        selected.push(label_id);
                    }
                }
            }
        }
    }

    match relationship_ops::set_labels(&ctx.conn, ctx.owner_id(), person_id, selected.clone()) {
        Ok(_) => {
            let names: Vec<String> = selected.iter().filter_map(|id| {
                all_labels.iter().find(|l| l.id == *id).map(|l| l.name.clone())
            }).collect();
            if names.is_empty() {
                println!("Labels: (none)");
            } else {
                println!("Labels: {}", names.join(", "));
            }
        }
        Err(e) => ctx.print_error(&e),
    }
}

fn select_circles(ctx: &CLIContext, person_id: Id<Person>) {
    let all_circles = circle_queries::active_circles(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    if all_circles.is_empty() {
        println!("No circles available.");
        return;
    }

    let current = ctx.circles_for(person_id);
    let mut selected: Vec<Id<Circle>> = current.iter().map(|c| c.id).collect();

    println!("Select circles (enter numbers to toggle, Enter when done):");

    loop {
        for (i, circle) in all_circles.iter().enumerate() {
            let marker = if selected.contains(&circle.id) { "[x]" } else { "[ ]" };
            println!("  {}. {} {}", i + 1, marker, circle.name);
        }

        let input = match ctx.prompt("Toggle (or Enter to finish): ") {
            Some(s) if s.is_empty() => break,
            Some(s) => s,
            None => break,
        };

        for token in input.split_whitespace() {
            if let Ok(idx) = token.parse::<usize>() {
                if idx > 0 && idx <= all_circles.len() {
                    let circle_id = all_circles[idx - 1].id;
                    if let Some(pos) = selected.iter().position(|id| *id == circle_id) {
                        selected.remove(pos);
                    } else {
                        selected.push(circle_id);
                    }
                }
            }
        }
    }

    // Compute diffs
    let current_ids: Vec<Id<Circle>> = current.iter().map(|c| c.id).collect();
    let to_add: Vec<Id<Person>> = vec![person_id];

    for circle_id in &selected {
        if !current_ids.contains(circle_id) {
            let _ = circle_ops::add_members(&ctx.conn, *circle_id, to_add.clone());
        }
    }
    for circle_id in &current_ids {
        if !selected.contains(circle_id) {
            let _ = circle_ops::remove_members(&ctx.conn, *circle_id, vec![person_id]);
        }
    }

    let names: Vec<String> = selected.iter().filter_map(|id| {
        all_circles.iter().find(|c| c.id == *id).map(|c| c.name.clone())
    }).collect();
    if names.is_empty() {
        println!("Circles: (none)");
    } else {
        println!("Circles: {}", names.join(", "));
    }
}

fn add_phones_loop(ctx: &CLIContext, person_id: Id<Person>) {
    loop {
        let number = match ctx.prompt("Phone number (or Enter to finish): ") {
            Some(s) if s.is_empty() => break,
            Some(s) => s,
            None => break,
        };
        let label = ctx.prompt("Label (optional): ").unwrap_or_default();
        let label_opt = if label.is_empty() { None } else { Some(label.as_str()) };
        match contact_ops::add_phone(&ctx.conn, person_id, &number, label_opt) {
            Ok(_) => println!("Added: {}", number),
            Err(e) => ctx.print_error(&e),
        }
    }
}

fn add_emails_loop(ctx: &CLIContext, person_id: Id<Person>) {
    loop {
        let email = match ctx.prompt("Email address (or Enter to finish): ") {
            Some(s) if s.is_empty() => break,
            Some(s) => s,
            None => break,
        };
        let label = ctx.prompt("Label (optional): ").unwrap_or_default();
        let label_opt = if label.is_empty() { None } else { Some(label.as_str()) };
        match contact_ops::add_email(&ctx.conn, person_id, &email, label_opt) {
            Ok(_) => println!("Added: {}", email),
            Err(e) => ctx.print_error(&e),
        }
    }
}

fn set_reminder_for(ctx: &CLIContext, person_id: Id<Person>) {
    let input = match ctx.prompt("Remind every how many days: ") {
        Some(s) => s,
        None => return,
    };
    match input.parse::<i32>() {
        Ok(days) if days > 0 => {
            match relationship_ops::set_reminder(&ctx.conn, person_id, Some(days)) {
                Ok(_) => println!("Reminder set for every {} days", days),
                Err(e) => ctx.print_error(&e),
            }
        }
        _ => println!("Invalid number, skipping reminder."),
    }
}

fn format_contacts(contacts: &[ContactEntry], type_name: &str) -> String {
    let filtered: Vec<&ContactEntry> = contacts.iter().filter(|c| {
        match type_name {
            "Phone" => c.contact_type == ContactType::Phone,
            "Email" => c.contact_type == ContactType::Email,
            _ => false,
        }
    }).collect();

    if filtered.is_empty() {
        "(none)".into()
    } else {
        filtered.iter().map(|entry| {
            let label = entry.label.as_ref().map(|l| format!(" ({})", l)).unwrap_or_default();
            let value = match &entry.value {
                ContactValue::StringValue { value } => value.clone(),
                ContactValue::AddressValue { value } => format!("{}, {}", value.street, value.city),
            };
            format!("{}{}", value, label)
        }).collect::<Vec<_>>().join(", ")
    }
}
