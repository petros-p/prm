use chrono::NaiveDate;

use crate::ai::llm_service::{self, CorrectionExample, ParsedInteraction};
use crate::cli::context::CLIContext;
use crate::db::correction_repo;
use crate::model::*;
use crate::ops::*;
use crate::queries::*;

pub fn ai_log(ctx: &CLIContext, args: &str) {
    if args.is_empty() {
        println!("Usage: ai-log <natural language description>");
        println!("Example: ai-log Had coffee with John at Starbucks, talked about his new job");
        return;
    }

    if let Err(err) = llm_service::check_ollama() {
        println!("Error: {}", err);
        return;
    }

    let known_names: Vec<String> = person_queries::active_people(&ctx.conn, ctx.owner_id())
        .unwrap_or_default()
        .into_iter()
        .filter(|p| !p.is_self)
        .map(|p| p.name)
        .collect();

    let corrections: Vec<CorrectionExample> =
        correction_repo::recent(&ctx.conn, ctx.owner_id(), 5)
            .unwrap_or_default()
            .into_iter()
            .map(|r| CorrectionExample {
                original_text: r.original_text,
                ai_output: r.ai_output,
                user_output: r.user_output,
            })
            .collect();

    println!("Parsing with AI (local)...");
    match llm_service::parse_interaction(args, &known_names, &corrections) {
        Err(err) => println!("Error: {}", err),
        Ok(parsed) => review_and_save(ctx, args, parsed),
    }
}

pub fn review_and_save(ctx: &CLIContext, original_text: &str, initial: ParsedInteraction) {
    let ai_original = initial.clone();
    let mut current = initial;

    loop {
        display_parsed(&current);
        println!();
        println!("Actions: (s)ave, (e)dit a field, (d)iscard");

        let choice = match ctx.prompt("Choice: ") {
            Some(c) => c.to_lowercase(),
            None => return,
        };

        match choice.as_str() {
            "s" | "save" => {
                maybe_save_correction(ctx, original_text, &ai_original, &current);
                save_interaction(ctx, &current);
                return;
            }
            "e" | "edit" => {
                current = edit_field(ctx, current);
            }
            "d" | "discard" => {
                println!("Discarded.");
                return;
            }
            _ => println!("Invalid choice. Enter 's' to save, 'e' to edit, or 'd' to discard."),
        }
    }
}

fn maybe_save_correction(
    ctx: &CLIContext,
    original_text: &str,
    ai_original: &ParsedInteraction,
    user_final: &ParsedInteraction,
) {
    let ai_json = match serde_json::to_string(ai_original) {
        Ok(j) => j,
        Err(_) => return,
    };
    let user_json = match serde_json::to_string(user_final) {
        Ok(j) => j,
        Err(_) => return,
    };
    if ai_json != user_json {
        let _ = correction_repo::insert(
            &ctx.conn,
            ctx.owner_id(),
            original_text,
            &ai_json,
            &user_json,
        );
    }
}

fn edit_field(ctx: &CLIContext, parsed: ParsedInteraction) -> ParsedInteraction {
    println!();
    println!("Which field to edit?");
    println!("  1. People");
    println!("  2. Medium");
    println!("  3. Location");
    println!("  4. Their location");
    println!("  5. Topics");
    println!("  6. Note");
    println!("  7. Date");

    let field = match ctx.prompt("Field (1-7): ") {
        Some(f) => f,
        None => return parsed,
    };

    let mut result = parsed;

    match field.as_str() {
        "1" => {
            let current = result.person_names.join(", ");
            let input = ctx
                .prompt(&format!("People (comma-separated) [{}]: ", current))
                .unwrap_or_default();
            if !input.is_empty() {
                let names: Vec<String> = input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !names.is_empty() {
                    result.person_names = names;
                }
            }
        }
        "2" => {
            println!("  1. In Person  2. Text  3. Phone Call  4. Video Call  5. Social Media");
            let input = ctx
                .prompt(&format!("Medium [{}]: ", format_medium(&result.medium)))
                .unwrap_or_default();
            result.medium = match input.as_str() {
                "1" => "InPerson".into(),
                "2" => "Text".into(),
                "3" => "PhoneCall".into(),
                "4" => "VideoCall".into(),
                "5" => "SocialMedia".into(),
                "" => result.medium,
                _ => {
                    println!("Invalid selection, keeping current.");
                    result.medium
                }
            };
        }
        "3" => {
            let current = if result.location.is_empty() {
                "(not set)"
            } else {
                &result.location
            };
            let input = ctx
                .prompt(&format!("Location [{}]: ", current))
                .unwrap_or_default();
            if !input.is_empty() {
                result.location = input;
            }
        }
        "4" => {
            if result.medium == "InPerson" {
                println!("Their location is the same as location for in-person interactions.");
            } else {
                let current = result
                    .their_location
                    .as_deref()
                    .unwrap_or("(none)");
                let input = ctx
                    .prompt(&format!("Their location [{}]: ", current))
                    .unwrap_or_default();
                if !input.is_empty() {
                    result.their_location = Some(input);
                }
            }
        }
        "5" => {
            let current = result.topics.join(", ");
            let input = ctx
                .prompt(&format!("Topics (comma-separated) [{}]: ", current))
                .unwrap_or_default();
            if !input.is_empty() {
                let new_topics: Vec<String> = input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !new_topics.is_empty() {
                    result.topics = new_topics;
                }
            }
        }
        "6" => {
            let current = result.note.as_deref().unwrap_or("(none)");
            let input = ctx
                .prompt(&format!("Note [{}]: ", current))
                .unwrap_or_default();
            if !input.is_empty() {
                result.note = Some(input);
            }
        }
        "7" => {
            let current = result.date.as_deref().unwrap_or("today");
            let input = ctx
                .prompt(&format!("Date (YYYY-MM-DD) [{}]: ", current))
                .unwrap_or_default();
            if !input.is_empty() {
                result.date = Some(input);
            }
        }
        _ => println!("Invalid field number."),
    }

    result
}

fn display_parsed(parsed: &ParsedInteraction) {
    println!();
    println!("Parsed interaction:");
    println!("  1. People:         {}", parsed.person_names.join(", "));
    println!("  2. Medium:         {}", format_medium(&parsed.medium));
    println!(
        "  3. Location:       {}",
        if parsed.location.is_empty() {
            "(not set)"
        } else {
            &parsed.location
        }
    );
    if parsed.medium == "InPerson" {
        println!("  4. Their location: (same as location)");
    } else {
        println!(
            "  4. Their location: {}",
            parsed.their_location.as_deref().unwrap_or("(not set)")
        );
    }
    println!("  5. Topics:         {}", parsed.topics.join(", "));
    println!(
        "  6. Note:           {}",
        parsed.note.as_deref().unwrap_or("(none)")
    );
    println!(
        "  7. Date:           {}",
        parsed.date.as_deref().unwrap_or("today")
    );
}

fn format_medium(s: &str) -> &str {
    match s {
        "InPerson" => "In Person",
        "Text" => "Text",
        "PhoneCall" => "Phone Call",
        "VideoCall" => "Video Call",
        "SocialMedia" => "Social Media",
        other => other,
    }
}

fn save_interaction(ctx: &CLIContext, parsed: &ParsedInteraction) {
    if parsed.location.is_empty() {
        println!("Error: Location is required. Please edit the location field first.");
        return;
    }

    let resolved_people: Vec<Person> = parsed
        .person_names
        .iter()
        .filter_map(|name| resolve_person(ctx, name))
        .collect();

    if resolved_people.is_empty() {
        println!("No people resolved. Interaction not saved.");
        return;
    }

    if resolved_people.len() < parsed.person_names.len() {
        let names: Vec<&str> = resolved_people.iter().map(|p| p.name.as_str()).collect();
        let input = ctx
            .prompt(&format!("Save interaction for: {}? (y/n): ", names.join(", ")))
            .unwrap_or_default()
            .to_lowercase();
        if input != "y" && input != "yes" {
            println!("Discarded.");
            return;
        }
    }

    let medium = parse_medium(&parsed.medium);
    let topics = parsed.topics.clone();
    let note = parsed.note.as_deref();
    let date = parsed
        .date
        .as_deref()
        .and_then(parse_date)
        .unwrap_or_else(CLIContext::today);

    for person in &resolved_people {
        let result = if medium == InteractionMedium::InPerson {
            interaction_ops::log_in_person(
                &ctx.conn,
                ctx.owner_id(),
                person.id,
                &parsed.location,
                topics.clone(),
                note,
                date,
            )
        } else {
            interaction_ops::log_remote(
                &ctx.conn,
                ctx.owner_id(),
                person.id,
                medium,
                &parsed.location,
                parsed.their_location.as_deref(),
                topics.clone(),
                note,
                date,
            )
        };

        match result {
            Ok(_) => println!("Logged interaction with {}", person.name),
            Err(e) => println!("Error logging interaction with {}: {}", person.name, e),
        }
    }
}

fn resolve_person(ctx: &CLIContext, name: &str) -> Option<Person> {
    let people = person_queries::active_people(&ctx.conn, ctx.owner_id()).unwrap_or_default();
    let lower = name.to_lowercase();
    let matches: Vec<&Person> = people
        .iter()
        .filter(|p| {
            p.name.to_lowercase().contains(&lower)
                || p.nickname
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&lower))
                    .unwrap_or(false)
        })
        .collect();

    match matches.len() {
        1 => Some(matches[0].clone()),
        n if n > 1 => {
            if let Some(exact) = matches.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
                return Some((*exact).clone());
            }
            println!("Multiple matches for '{}':", name);
            for p in &matches {
                println!("  {}", p.name);
            }
            println!("Please edit the people field to be more specific.");
            None
        }
        _ => {
            println!("'{}' is not in your network.", name);
            let answer = ctx
                .prompt(&format!("Would you like to add {}? (y/n): ", name))
                .unwrap_or_default()
                .to_lowercase();
            if answer == "y" || answer == "yes" {
                match person_ops::add_person(&ctx.conn, ctx.owner_id(), name, None, None, None, None, None) {
                    Ok(person) => {
                        println!("Added {} to your network.", name);
                        Some(person)
                    }
                    Err(e) => {
                        println!("Error adding person: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        }
    }
}

fn parse_medium(s: &str) -> InteractionMedium {
    match s {
        "InPerson" => InteractionMedium::InPerson,
        "Text" => InteractionMedium::Text,
        "PhoneCall" => InteractionMedium::PhoneCall,
        "VideoCall" => InteractionMedium::VideoCall,
        "SocialMedia" => InteractionMedium::SocialMedia,
        other => {
            println!("Unknown medium '{}', defaulting to In Person", other);
            InteractionMedium::InPerson
        }
    }
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        Ok(d) => Some(d),
        Err(_) => {
            println!("Could not parse date '{}', using today", s);
            None
        }
    }
}
