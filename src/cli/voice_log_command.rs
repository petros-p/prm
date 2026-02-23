use std::path::Path;

use crate::ai::{llm_service::{self, CorrectionExample}, whisper_service};
use crate::cli::ai_log_command;
use crate::cli::context::CLIContext;
use crate::db::correction_repo;
use crate::queries::person_queries;

pub fn voice_log(ctx: &CLIContext, args: &str) {
    if args.is_empty() {
        println!("Usage: voice-log <wav-file>");
        println!("Example: voice-log recording.wav");
        return;
    }

    let wav_path = Path::new(args);
    if !wav_path.exists() {
        println!("Error: File not found: {}", args);
        return;
    }

    if let Err(err) = llm_service::check_ollama() {
        println!("Error: {}", err);
        return;
    }

    println!("Transcribing audio (local, via Whisper)...");
    let transcript = match whisper_service::transcribe(wav_path) {
        Ok(t) => t,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    if transcript.is_empty() {
        println!("No speech detected in the audio file.");
        return;
    }

    println!();
    println!("Transcript: {}", transcript);
    println!();

    let text = match ctx.prompt("Edit transcript before parsing (or press Enter to use as-is): ") {
        Some(edited) if !edited.trim().is_empty() => edited,
        _ => transcript,
    };

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
    match llm_service::parse_interaction(&text, &known_names, &corrections) {
        Err(err) => println!("Error: {}", err),
        Ok(parsed) => ai_log_command::review_and_save(ctx, &text, parsed),
    }
}
