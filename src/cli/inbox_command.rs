use std::fs;
use std::path::{Path, PathBuf};

use crate::ai::llm_service::{self, CorrectionExample};
use crate::cli::ai_log_command;
use crate::cli::context::CLIContext;
use crate::db::correction_repo;
use crate::queries::person_queries;

pub fn inbox(ctx: &CLIContext, _args: &str) {
    let pending = match pending_files(&ctx.inbox_dir) {
        Ok(files) => files,
        Err(e) => {
            println!("Error reading inbox folder {}: {}", ctx.inbox_dir.display(), e);
            return;
        }
    };

    if pending.is_empty() {
        println!(
            "No notes waiting. Drop a .txt file into {} to capture one.",
            ctx.inbox_dir.display()
        );
        return;
    }

    if let Err(err) = llm_service::check_ollama() {
        println!("Error: {}", err);
        return;
    }

    println!(
        "{} note{} waiting in your inbox.",
        pending.len(),
        if pending.len() == 1 { "" } else { "s" }
    );

    for path in pending {
        process_file(ctx, &path);
    }
}

/// Count pending inbox files without processing them, for the startup hint.
pub fn pending_count(dir: &Path) -> usize {
    pending_files(dir).map(|f| f.len()).unwrap_or(0)
}

fn process_file(ctx: &CLIContext, path: &Path) {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            println!("Skipping {} ({}) — moved to 'failed'.", path.display(), e);
            move_to_subdir(ctx, path, "failed");
            return;
        }
    };

    let text = text.trim();
    if text.is_empty() {
        println!("Skipping {} (empty).", path.display());
        move_to_subdir(ctx, path, "processed");
        return;
    }

    println!();
    println!("--- {} ---", path.display());
    println!("{}", text);
    println!();

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
    match llm_service::parse_interaction(text, &known_names, &corrections) {
        Err(err) => println!("Error: {}", err),
        Ok(parsed) => ai_log_command::review_and_save(ctx, text, parsed),
    }

    move_to_subdir(ctx, path, "processed");
}

fn pending_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            ext == "txt" || ext == "md"
        })
        .collect();
    files.sort();
    Ok(files)
}

fn move_to_subdir(ctx: &CLIContext, path: &Path, subdir: &str) {
    let target_dir = ctx.inbox_dir.join(subdir);
    if let Err(e) = fs::create_dir_all(&target_dir) {
        println!("Warning: could not create '{}' folder: {}", subdir, e);
        return;
    }
    let Some(file_name) = path.file_name() else {
        return;
    };
    let dest = target_dir.join(file_name);
    if let Err(e) = fs::rename(path, &dest) {
        println!("Warning: could not move {} to '{}': {}", path.display(), subdir, e);
    }
}
