use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut db_path: Option<PathBuf> = None;
    let mut import_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" | "-f" => {
                db_path = args.next().map(PathBuf::from);
                if db_path.is_none() {
                    eprintln!("Error: --file requires a path argument");
                    std::process::exit(1);
                }
            }
            "--import" => {
                import_path = args.next().map(PathBuf::from);
                if import_path.is_none() {
                    eprintln!("Error: --import requires a JSON file path");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("PRM - Personal Relationship Manager");
                println!();
                println!("Usage: prm [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -f, --file <PATH>      Database file path (default: .data/prm.db)");
                println!("  --import <JSON_PATH>   Import data from Scala PRM JSON file");
                println!("  -h, --help             Show this help");
                return;
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                eprintln!("Use --help for usage information.");
                std::process::exit(1);
            }
        }
    }

    let db_path = db_path.unwrap_or_else(|| {
        let dir = PathBuf::from(".data");
        if !dir.exists() {
            std::fs::create_dir_all(&dir).expect("Failed to create .data directory");
        }
        dir.join("prm.db")
    });

    if let Some(json_path) = import_path {
        println!("Importing from {}...", json_path.display());
        if db_path.exists() {
            eprintln!("Error: Database file {} already exists.", db_path.display());
            eprintln!("Remove it first or use --file to specify a different path.");
            std::process::exit(1);
        }
        match prm::migrate::import_json(&json_path, &db_path) {
            Ok(stats) => {
                println!("Import complete!");
                println!("  People: {}", stats.people);
                println!("  Relationships: {}", stats.relationships);
                println!("  Interactions: {}", stats.interactions);
                println!("  Circles: {}", stats.circles);
                println!("  Labels: {}", stats.labels);
                println!("  Custom contact types: {}", stats.custom_contact_types);
            }
            Err(e) => {
                eprintln!("Import failed: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    prm::cli::run(&db_path);
}
