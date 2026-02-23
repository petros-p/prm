use crate::cli::context::CLIContext;
use crate::model::*;
use crate::ops::*;
use crate::queries::*;

pub fn log(ctx: &CLIContext, args: &str) {
    match if args.is_empty() { println!("Usage: log <name>"); return; } else { ctx.find_person(args) } {
        Some(person) => log_for_person(ctx, &person),
        None => {}
    }
}

pub fn log_for_person(ctx: &CLIContext, person: &Person) {
    println!("Logging interaction with {}", person.name);

    println!("How did you interact?");
    for (i, medium) in InteractionMedium::ALL.iter().enumerate() {
        println!("  {}. {}", i + 1, medium.display_name());
    }

    let medium_input = ctx.prompt("Medium (1-5): ").unwrap_or_default();
    let medium = medium_input
        .parse::<usize>()
        .ok()
        .and_then(|i| InteractionMedium::ALL.get(i.wrapping_sub(1)))
        .copied()
        .unwrap_or_else(|| {
            println!("Invalid selection, defaulting to In Person");
            InteractionMedium::InPerson
        });

    let (my_location, their_location) = if medium == InteractionMedium::InPerson {
        let default_loc = person.location.as_ref().map(|l| format!(" [{}]", l)).unwrap_or_default();
        let loc_input = ctx.prompt(&format!("Location{}: ", default_loc)).unwrap_or_default();
        let loc = if loc_input.is_empty() {
            person.location.clone().unwrap_or_default()
        } else {
            loc_input
        };
        if loc.is_empty() {
            println!("Location is required.");
            return;
        }
        (loc.clone(), Some(loc))
    } else {
        let my_loc = match ctx.prompt("Your location: ") {
            Some(s) if !s.is_empty() => s,
            _ => { println!("Your location is required."); return; }
        };

        let default_loc = person.location.as_ref().map(|l| format!(" [{}]", l)).unwrap_or_default();
        let their_input = ctx.prompt(&format!("Their location (optional){}: ", default_loc)).unwrap_or_default();
        let their_loc = if their_input.is_empty() {
            person.location.clone()
        } else {
            Some(their_input)
        };
        (my_loc, their_loc)
    };

    let topics_input = ctx.prompt("Topics (comma-separated): ").unwrap_or_default();
    let topics: Vec<String> = topics_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if topics.is_empty() {
        println!("At least one topic is required.");
        return;
    }

    let note = ctx.prompt("Note (optional): ").unwrap_or_default();
    let note_opt = if note.is_empty() { None } else { Some(note.as_str()) };
    let today = CLIContext::today();

    let result = if medium == InteractionMedium::InPerson {
        interaction_ops::log_in_person(
            &ctx.conn, ctx.owner_id(), person.id, &my_location, topics, note_opt, today,
        )
    } else {
        interaction_ops::log_remote(
            &ctx.conn, ctx.owner_id(), person.id, medium, &my_location,
            their_location.as_deref(), topics, note_opt, today,
        )
    };

    match result {
        Ok(_) => println!("Logged interaction with {}", person.name),
        Err(e) => ctx.print_error(&e),
    }
}

pub fn show_reminders(ctx: &CLIContext) {
    const DUE_SOON_DAYS: i64 = 7;
    let today = CLIContext::today();
    let all = reminder_queries::all_reminders(&ctx.conn, ctx.owner_id(), today).unwrap_or_default();

    let overdue: Vec<_> = all.iter().filter(|s| match &s.overdue_status {
        reminder_queries::OverdueStatus::NeverContacted => true,
        reminder_queries::OverdueStatus::DaysOverdue(d) => *d > 0,
    }).collect();

    let due_soon: Vec<_> = all.iter().filter(|s| match &s.overdue_status {
        reminder_queries::OverdueStatus::NeverContacted => false,
        reminder_queries::OverdueStatus::DaysOverdue(d) => *d <= 0 && *d > -DUE_SOON_DAYS,
    }).collect();

    if overdue.is_empty() && due_soon.is_empty() {
        println!("No reminders due. You're all caught up.");
        return;
    }

    if !overdue.is_empty() {
        println!("Overdue ({}):", overdue.len());
        for status in &overdue {
            let detail = match &status.overdue_status {
                reminder_queries::OverdueStatus::NeverContacted => "never contacted".into(),
                reminder_queries::OverdueStatus::DaysOverdue(d) => {
                    let last = status.days_since_last_interaction
                        .map(|d| format!("last contact {}", CLIContext::format_days_ago(d)))
                        .unwrap_or_default();
                    format!("{} days overdue ({})", d, last)
                }
            };
            println!("  {} — {}", status.person.name, detail);
        }
    }

    if !due_soon.is_empty() {
        if !overdue.is_empty() { println!(); }
        println!("Due soon ({}):", due_soon.len());
        for status in &due_soon {
            if let reminder_queries::OverdueStatus::DaysOverdue(d) = &status.overdue_status {
                let days_until = -d;
                let last = status.days_since_last_interaction
                    .map(|d| format!(", last contact {}", CLIContext::format_days_ago(d)))
                    .unwrap_or_default();
                println!("  {} — due in {} day{}{}",
                    status.person.name,
                    days_until,
                    if days_until == 1 { "" } else { "s" },
                    last,
                );
            }
        }
    }
}

pub fn set_reminder(ctx: &CLIContext, args: &str) {
    let person = match if args.is_empty() { println!("Usage: set-reminder <name>"); return; } else { ctx.find_person(args) } {
        Some(p) => p,
        None => return,
    };

    let input = match ctx.prompt("Remind every how many days? (0 to remove reminder): ") {
        Some(s) => s,
        None => return,
    };

    match input.parse::<i32>() {
        Ok(0) => {
            match relationship_ops::set_reminder(&ctx.conn, person.id, None) {
                Ok(_) => println!("Reminder removed for {}", person.name),
                Err(e) => ctx.print_error(&e),
            }
        }
        Ok(days) if days > 0 => {
            match relationship_ops::set_reminder(&ctx.conn, person.id, Some(days)) {
                Ok(_) => println!("Reminder set: reach out to {} every {} days", person.name, days),
                Err(e) => ctx.print_error(&e),
            }
        }
        _ => println!("Invalid number"),
    }
}

pub fn print_stats(ctx: &CLIContext) {
    match stats_queries::stats(&ctx.conn, ctx.owner_id(), ctx.self_id) {
        Ok(s) => {
            println!();
            println!("People: {} active, {} archived", s.active_people, s.archived_people);
            if s.never_contacted > 0 {
                println!("  Never contacted: {}", s.never_contacted);
            }
            if s.no_reminder_set > 0 {
                println!("  No reminder set: {}", s.no_reminder_set);
            }
            println!("Interactions: {}", s.total_interactions);
            if let Some((name, days)) = &s.longest_gap {
                println!("  Longest gap: {} ({})", name, CLIContext::format_days_ago(*days));
            }
            if s.active_circles > 0 {
                println!("Circles: {} active, {} archived", s.active_circles, s.archived_circles);
            }
            if s.reminders_overdue > 0 {
                println!("Reminders overdue: {}", s.reminders_overdue);
            }
            println!();
        }
        Err(e) => ctx.print_error(&e),
    }
}
