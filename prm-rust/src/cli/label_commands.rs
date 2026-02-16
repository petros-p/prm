use crate::cli::context::CLIContext;
use crate::model::*;
use crate::ops::*;
use crate::queries::*;

pub fn list(ctx: &CLIContext) {
    let labels = relationship_queries::active_labels(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    if labels.is_empty() {
        println!("No labels yet. Use 'add-label <name>' to create one.");
    } else {
        println!("Labels ({}):", labels.len());
        for label in &labels {
            let count = relationship_queries::people_with_label(&ctx.conn, ctx.owner_id(), label.id)
                .map(|p| p.len())
                .unwrap_or(0);
            println!("  {} ({})", label.name, count);
        }
    }
}

pub fn show(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: show-label <name>"); return; } else { ctx.find_label(args) } {
        Some(label) => {
            println!();
            println!("Label: {}", label.name);
            println!("Archived: {}", if label.archived { "yes" } else { "no" });

            let people = relationship_queries::people_with_label(&ctx.conn, ctx.owner_id(), label.id).unwrap_or_default();
            if people.is_empty() {
                println!("People: (none)");
            } else {
                let names: Vec<&str> = people.iter().map(|p| p.name.as_str()).collect();
                println!("People: {}", names.join(", "));
            }
            println!();
        }
        None => {}
    }
}

pub fn add(ctx: &CLIContext, args: &str) {
    let name = if !args.is_empty() {
        args.to_string()
    } else {
        match ctx.prompt("Label name: ") {
            Some(s) if !s.is_empty() => s,
            _ => { println!("Name cannot be empty."); return; }
        }
    };

    match label_ops::add_label(&ctx.conn, ctx.owner_id(), &name) {
        Ok(label) => println!("Created label: {}", label.name),
        Err(e) => ctx.print_error(&e),
    }
}

pub fn edit(ctx: &CLIContext, args: &str) {
    let label = match if args.is_empty() { println!("Usage: edit-label <name>"); return; } else { ctx.find_label(args) } {
        Some(l) => l,
        None => return,
    };

    println!("Editing label: {}", label.name);
    println!();
    println!("What would you like to edit?");
    println!("  1. Name");
    println!("  2. People with this label");
    println!("  3. Both");
    println!();

    match ctx.prompt("Choice (1-3, or Enter to cancel): ").as_deref() {
        Some("1") => edit_name(ctx, &label),
        Some("2") => edit_people(ctx, &label),
        Some("3") => {
            edit_name(ctx, &label);
            // Re-fetch label after name change
            if let Some(updated) = crate::db::relationship_repo::find_label_by_id(&ctx.conn, label.id).ok().flatten() {
                edit_people(ctx, &updated);
            }
        }
        Some("") => println!("Cancelled."),
        _ => println!("Invalid choice."),
    }
}

fn edit_name(ctx: &CLIContext, label: &RelationshipLabel) {
    let input = match ctx.prompt(&format!("Name [{}]: ", label.name)) {
        Some(s) if !s.is_empty() && s != label.name => s,
        _ => return,
    };
    match label_ops::update_label(&ctx.conn, ctx.owner_id(), label.id, Some(&input)) {
        Ok(_) => println!("Renamed label to: {}", input),
        Err(e) => ctx.print_error(&e),
    }
}

fn edit_people(ctx: &CLIContext, label: &RelationshipLabel) {
    let all_people: Vec<Person> = person_queries::active_people(&ctx.conn, ctx.owner_id())
        .unwrap_or_default()
        .into_iter()
        .filter(|p| !p.is_self)
        .collect();

    if all_people.is_empty() {
        println!("No people to assign labels to.");
        return;
    }

    let current = relationship_queries::people_with_label(&ctx.conn, ctx.owner_id(), label.id).unwrap_or_default();
    let mut selected: Vec<Id<Person>> = current.iter().map(|p| p.id).collect();

    println!();
    println!("Toggle people with this label (enter numbers, Enter when done):");

    loop {
        for (i, person) in all_people.iter().enumerate() {
            let marker = if selected.contains(&person.id) { "[x]" } else { "[ ]" };
            println!("  {}. {} {}", i + 1, marker, person.name);
        }

        let input = match ctx.prompt("Toggle (or Enter to finish): ") {
            Some(s) if s.is_empty() => break,
            Some(s) => s,
            None => break,
        };

        for token in input.split_whitespace() {
            if let Ok(idx) = token.parse::<usize>() {
                if idx > 0 && idx <= all_people.len() {
                    let pid = all_people[idx - 1].id;
                    if let Some(pos) = selected.iter().position(|id| *id == pid) {
                        selected.remove(pos);
                    } else {
                        selected.push(pid);
                    }
                }
            }
        }
    }

    // Compute diffs
    let current_ids: Vec<Id<Person>> = current.iter().map(|p| p.id).collect();
    let to_add: Vec<Id<Person>> = selected.iter().filter(|id| !current_ids.contains(id)).copied().collect();
    let to_remove: Vec<Id<Person>> = current_ids.iter().filter(|id| !selected.contains(id)).copied().collect();

    for person_id in to_add {
        let _ = relationship_ops::add_labels(&ctx.conn, ctx.owner_id(), person_id, vec![label.id]);
    }
    for person_id in to_remove {
        let _ = relationship_ops::remove_labels(&ctx.conn, ctx.owner_id(), person_id, vec![label.id]);
    }

    println!("Updated: {} people now have label '{}'", selected.len(), label.name);
}

pub fn archive(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: archive-label <name>"); return; } else { ctx.find_label(args) } {
        Some(label) => match label_ops::archive_label(&ctx.conn, label.id) {
            Ok(_) => println!("Archived label: {}", label.name),
            Err(e) => ctx.print_error(&e),
        },
        None => {}
    }
}

pub fn unarchive(ctx: &CLIContext, args: &str) {
    if args.is_empty() {
        println!("Usage: unarchive-label <name>");
        return;
    }

    let archived = relationship_queries::archived_labels(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    let lower = args.to_lowercase();
    let matches: Vec<&RelationshipLabel> = archived.iter().filter(|l| l.name.to_lowercase().contains(&lower)).collect();

    match matches.len() {
        0 => println!("No archived label found matching '{}'", args),
        1 => match label_ops::unarchive_label(&ctx.conn, matches[0].id) {
            Ok(_) => println!("Restored label: {}", matches[0].name),
            Err(e) => ctx.print_error(&e),
        },
        _ => {
            println!("Multiple matches found:");
            for l in &matches { println!("  {}", l.name); }
            println!("Please be more specific.");
        }
    }
}

pub fn list_archived(ctx: &CLIContext) {
    let archived = relationship_queries::archived_labels(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    if archived.is_empty() {
        println!("No archived labels.");
    } else {
        println!("Archived labels ({}):", archived.len());
        for l in &archived { println!("  {}", l.name); }
    }
}
