use crate::cli::context::CLIContext;
use crate::model::*;
use crate::ops::*;
use crate::queries::*;

pub fn list(ctx: &CLIContext) {
    let circles = circle_queries::active_circles(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    if circles.is_empty() {
        println!("No circles yet. Use 'add-circle <name>' to create one.");
    } else {
        println!("Circles ({}):", circles.len());
        for circle in &circles {
            println!("  {} ({} members)", circle.name, circle.member_ids.len());
        }
    }
}

pub fn add(ctx: &CLIContext, args: &str) {
    let name = if !args.is_empty() {
        args.to_string()
    } else {
        match ctx.prompt("Circle name: ") {
            Some(s) if !s.is_empty() => s,
            _ => { println!("Name cannot be empty."); return; }
        }
    };

    let desc = ctx.prompt("Description (optional): ").unwrap_or_default();
    let desc_opt = if desc.is_empty() { None } else { Some(desc.as_str()) };

    println!("Add members (enter numbers separated by spaces, or press Enter to skip):");
    let all_people = person_queries::active_people(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    let people: Vec<_> = all_people.iter().filter(|p| p.id != ctx.self_id).collect();
    for (i, person) in people.iter().enumerate() {
        println!("  {}. {}", i + 1, person.name);
    }

    let member_input = ctx.prompt("Members: ").unwrap_or_default();
    let member_ids: Vec<Id<Person>> = member_input
        .split_whitespace()
        .filter_map(|s| s.parse::<usize>().ok())
        .filter_map(|i| people.get(i.wrapping_sub(1)).map(|p| p.id))
        .collect();

    match circle_ops::create_circle(&ctx.conn, ctx.owner_id(), &name, desc_opt, member_ids.clone()) {
        Ok(circle) => println!("Created circle: {} with {} members", circle.name, member_ids.len()),
        Err(e) => ctx.print_error(&e),
    }
}

pub fn show(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: show-circle <name>"); return; } else { ctx.find_circle(args) } {
        Some(circle) => {
            println!();
            println!("Name: {}", circle.name);
            println!("Description: {}", circle.description.as_deref().unwrap_or("(none)"));
            println!("Archived: {}", if circle.archived { "yes" } else { "no" });

            let all_members = circle_queries::circle_members(&ctx.conn, circle.id).unwrap_or_default();
            let members: Vec<_> = all_members.iter().filter(|p| p.id != ctx.self_id).collect();
            if members.is_empty() {
                println!("Members: (none)");
            } else {
                let names: Vec<&str> = members.iter().map(|p| p.name.as_str()).collect();
                println!("Members: {}", names.join(", "));
            }
            println!();
        }
        None => {}
    }
}

pub fn edit(ctx: &CLIContext, args: &str) {
    let circle = match if args.is_empty() { println!("Usage: edit-circle <name>"); return; } else { ctx.find_circle(args) } {
        Some(c) => c,
        None => return,
    };

    println!("Editing circle: {}", circle.name);
    println!();

    // Edit name
    if let Some(name_input) = ctx.prompt(&format!("Name [{}]: ", circle.name)) {
        if !name_input.is_empty() {
            match circle_ops::update_circle(&ctx.conn, circle.id, Some(&name_input), None) {
                Ok(_) => {}
                Err(e) => ctx.print_error(&e),
            }
        }
    }

    // Edit description
    let current_desc = circle.description.as_deref().unwrap_or("");
    if let Some(desc_input) = ctx.prompt(&format!("Description [{}] ('clear' to remove): ", current_desc)) {
        if !desc_input.is_empty() {
            let new_desc = if desc_input.eq_ignore_ascii_case("clear") { None } else { Some(desc_input.as_str()) };
            match circle_ops::update_circle(&ctx.conn, circle.id, None, Some(new_desc)) {
                Ok(_) => {}
                Err(e) => ctx.print_error(&e),
            }
        }
    }

    // Edit members
    let all_people = person_queries::active_people(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    let current_members: Vec<Id<Person>> = circle.member_ids.clone();
    let mut selected = current_members.clone();

    println!();
    println!("Edit members (enter numbers to toggle, press Enter when done):");

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

    match crate::db::circle_repo::set_members(&ctx.conn, circle.id, &selected) {
        Ok(_) => println!("Circle updated with {} members", selected.len()),
        Err(e) => ctx.print_error(&e),
    }
}

pub fn archive(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: archive-circle <name>"); return; } else { ctx.find_circle(args) } {
        Some(circle) => match circle_ops::archive_circle(&ctx.conn, circle.id) {
            Ok(_) => println!("Archived circle: {}", circle.name),
            Err(e) => ctx.print_error(&e),
        },
        None => {}
    }
}

pub fn unarchive(ctx: &CLIContext, args: &str) {
    if args.is_empty() {
        println!("Usage: unarchive-circle <name>");
        return;
    }

    let archived = circle_queries::archived_circles(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    let lower = args.to_lowercase();
    let matches: Vec<&Circle> = archived.iter().filter(|c| c.name.to_lowercase().contains(&lower)).collect();

    match matches.len() {
        0 => println!("No archived circle found matching '{}'", args),
        1 => match circle_ops::unarchive_circle(&ctx.conn, matches[0].id) {
            Ok(_) => println!("Restored circle: {}", matches[0].name),
            Err(e) => ctx.print_error(&e),
        },
        _ => {
            println!("Multiple matches found:");
            for c in &matches { println!("  {}", c.name); }
            println!("Please be more specific.");
        }
    }
}

pub fn list_archived(ctx: &CLIContext) {
    let archived = circle_queries::archived_circles(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    if archived.is_empty() {
        println!("No archived circles.");
    } else {
        println!("Archived circles ({}):", archived.len());
        for c in &archived { println!("  {}", c.name); }
    }
}
