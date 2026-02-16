use chrono::Local;
use rusqlite::Connection;
use std::io::{self, Write};

use crate::db::contact_repo;
use crate::model::*;
use crate::queries::*;

pub struct CLIContext {
    pub conn: Connection,
    pub user: User,
    pub self_id: Id<Person>,
}

impl CLIContext {
    pub fn new(conn: Connection, user: User, self_id: Id<Person>) -> Self {
        Self { conn, user, self_id }
    }

    pub fn owner_id(&self) -> Id<User> {
        self.user.id
    }

    /// Prompt and read a line from stdin. Returns None on EOF.
    pub fn read_line(&self, prompt: &str) -> Option<String> {
        print!("{}", prompt);
        io::stdout().flush().ok();
        let mut buf = String::new();
        match io::stdin().read_line(&mut buf) {
            Ok(0) => None,
            Ok(_) => Some(buf.trim_end_matches('\n').trim_end_matches('\r').to_string()),
            Err(_) => None,
        }
    }

    /// Read a line, trimmed.
    pub fn prompt(&self, prompt: &str) -> Option<String> {
        self.read_line(prompt).map(|s| s.trim().to_string())
    }

    /// Find an active person by name query. Prints error if not found or ambiguous.
    pub fn find_person(&self, args: &str) -> Option<Person> {
        let query = args.trim();
        if query.is_empty() {
            return None;
        }

        let people = person_queries::active_people(&self.conn, self.owner_id()).unwrap_or_default();
        let lower = query.to_lowercase();
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
            0 => {
                println!("No person found matching '{}'", query);
                None
            }
            1 => Some(matches[0].clone()),
            _ => {
                // Check for exact match
                if let Some(exact) = matches.iter().find(|p| p.name.eq_ignore_ascii_case(query)) {
                    return Some((*exact).clone());
                }
                println!("Multiple matches found:");
                for p in &matches {
                    println!("  {}", p.name);
                }
                println!("Please be more specific.");
                None
            }
        }
    }

    /// Find an active circle by name query.
    pub fn find_circle(&self, args: &str) -> Option<Circle> {
        let query = args.trim();
        if query.is_empty() {
            return None;
        }

        let circles = circle_queries::active_circles(&self.conn, self.owner_id()).unwrap_or_default();
        let lower = query.to_lowercase();
        let matches: Vec<&Circle> = circles
            .iter()
            .filter(|c| c.name.to_lowercase().contains(&lower))
            .collect();

        match matches.len() {
            0 => {
                println!("No circle found matching '{}'", query);
                None
            }
            1 => Some(matches[0].clone()),
            _ => {
                if let Some(exact) = matches.iter().find(|c| c.name.eq_ignore_ascii_case(query)) {
                    return Some((*exact).clone());
                }
                println!("Multiple matches found:");
                for c in &matches {
                    println!("  {}", c.name);
                }
                println!("Please be more specific.");
                None
            }
        }
    }

    /// Find an active label by name query.
    pub fn find_label(&self, args: &str) -> Option<RelationshipLabel> {
        let query = args.trim();
        if query.is_empty() {
            return None;
        }

        let labels = relationship_queries::active_labels(&self.conn, self.owner_id()).unwrap_or_default();
        let lower = query.to_lowercase();
        let matches: Vec<&RelationshipLabel> = labels
            .iter()
            .filter(|l| l.name.to_lowercase().contains(&lower))
            .collect();

        match matches.len() {
            0 => {
                println!("No label found matching '{}'", query);
                None
            }
            1 => Some(matches[0].clone()),
            _ => {
                if let Some(exact) = matches.iter().find(|l| l.name.eq_ignore_ascii_case(query)) {
                    return Some((*exact).clone());
                }
                println!("Multiple matches found:");
                for l in &matches {
                    println!("  {}", l.name);
                }
                println!("Please be more specific.");
                None
            }
        }
    }

    pub fn format_days_ago(days: i64) -> String {
        match days {
            0 => "today".into(),
            1 => "yesterday".into(),
            n if n < 7 => format!("{} days ago", n),
            n if n < 30 => format!("{} week(s) ago", n / 7),
            n if n < 365 => format!("{} month(s) ago", n / 30),
            n => format!("{} year(s) ago", n / 365),
        }
    }

    pub fn today() -> chrono::NaiveDate {
        Local::now().date_naive()
    }

    /// Get contacts for a person.
    pub fn contacts_for(&self, person_id: Id<Person>) -> Vec<ContactEntry> {
        contact_repo::find_by_person(&self.conn, person_id).unwrap_or_default()
    }

    /// Get labels for a person.
    pub fn labels_for(&self, person_id: Id<Person>) -> Vec<RelationshipLabel> {
        relationship_queries::labels_for(&self.conn, person_id).unwrap_or_default()
    }

    /// Get circles for a person.
    pub fn circles_for(&self, person_id: Id<Person>) -> Vec<Circle> {
        circle_queries::circles_for_person(&self.conn, self.owner_id(), person_id).unwrap_or_default()
    }

    /// Print an error.
    pub fn print_error(&self, e: &crate::error::PrmError) {
        println!("Error: {}", e);
    }
}
