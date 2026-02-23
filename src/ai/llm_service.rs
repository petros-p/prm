use std::env;
use serde::{Deserialize, Serialize};

const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
const DEFAULT_MODEL: &str = "llama3.2:3b";

/// A past correction: what the AI parsed vs what the user actually saved.
pub struct CorrectionExample {
    pub original_text: String,
    pub ai_output: String,
    pub user_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedInteraction {
    #[serde(default, alias = "personName")]
    pub person_names: Vec<String>,
    #[serde(default = "default_medium")]
    pub medium: String,
    #[serde(default)]
    pub location: String,
    pub their_location: Option<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    pub note: Option<String>,
    pub date: Option<String>,
}

fn default_medium() -> String {
    "InPerson".into()
}

fn ollama_url() -> String {
    env::var("OLLAMA_HOST").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string())
}

fn ollama_model() -> String {
    env::var("PRM_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string())
}

/// Check that Ollama is running and reachable.
pub fn check_ollama() -> Result<(), String> {
    let url = ollama_url();
    ureq::get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .call()
        .map_err(|_| format!(
            "Cannot connect to Ollama at {}. Is it running?\n  Install: https://ollama.ai\n  Start:   ollama serve",
            url
        ))?;
    Ok(())
}

pub fn parse_interaction(
    input: &str,
    known_names: &[String],
    corrections: &[CorrectionExample],
) -> Result<ParsedInteraction, String> {
    let names_str = known_names.join(", ");
    let today = chrono::Local::now().format("%Y-%m-%d");
    let model = ollama_model();

    let corrections_block = if corrections.is_empty() {
        String::new()
    } else {
        let mut block = String::from("\nPast corrections to learn from (most recent first):\n");
        for (i, c) in corrections.iter().enumerate() {
            block.push_str(&format!(
                "\nExample {}:\nInput: {}\nYou parsed: {}\nUser corrected to: {}\n",
                i + 1,
                c.original_text,
                c.ai_output,
                c.user_output,
            ));
        }
        block.push_str("\nApply these learnings when parsing the new input.\n");
        block
    };

    let system_prompt = format!(
        r#"You extract interaction metadata from natural language descriptions.
Today's date is {today}.
Known contacts: [{names_str}]
Respond with JSON only, no other text.
JSON schema: {{ "personNames": ["..."], "medium": "InPerson|Text|PhoneCall|VideoCall|SocialMedia", "location": "...", "theirLocation": null, "topics": ["..."], "note": null, "date": null }}
Rules:
- personNames is an array of ALL people mentioned in the input. If multiple people are mentioned, include all of them. Use names exactly as written. Match to known contacts when possible. NEVER substitute different names.
- If the medium is not mentioned, default to "InPerson"
- location: use the most specific location from the input (e.g. "Charlton, MA" not just "home"). Include the full place name, city, or address as given.
- theirLocation is only for remote interactions where their location differs; set to null for in-person
- topics: ONLY include activities or subjects explicitly mentioned in the input. Do NOT infer or add topics that weren't stated. Be aware of slang (e.g. "gas" means great/amazing, not cooking).
- note is for any additional context not captured in other fields; set to null if none
- date: ONLY set to a "YYYY-MM-DD" string if the user explicitly mentions a specific date (e.g. "yesterday", "last Friday", "on March 5th"). Otherwise MUST be null. null means today.{corrections_block}"#
    );

    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": input }
        ],
        "format": "json",
        "stream": false
    });

    let api_url = format!("{}/api/chat", ollama_url());

    let response = ureq::post(&api_url)
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(120))
        .send_json(request_body)
        .map_err(|e| match e {
            ureq::Error::Status(code, resp) => {
                let body = resp.into_string().unwrap_or_default();
                format!("Ollama request failed (HTTP {}): {}", code, &body[..body.len().min(200)])
            }
            ureq::Error::Transport(t) => {
                if t.to_string().contains("timed out") {
                    "Request timed out. Local models can be slow on first run — try again.".into()
                } else {
                    format!("Could not connect to Ollama: {}", t)
                }
            }
        })?;

    let json: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    let content = json
        .pointer("/message/content")
        .and_then(|v| v.as_str())
        .ok_or("No content in Ollama response")?;

    parse_llm_json(content)
}

// Custom deserialization from the LLM JSON which uses camelCase
fn parse_llm_json(content: &str) -> Result<ParsedInteraction, String> {
    let json: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    let person_names = match json.get("personNames") {
        Some(v) if v.is_array() => v
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .filter(|s| !s.is_empty())
            .collect(),
        _ => {
            // Fallback to single personName
            match json.get("personName").and_then(|v| v.as_str()) {
                Some(name) if !name.is_empty() => vec![name.to_string()],
                _ => Vec::new(),
            }
        }
    };

    let medium = json
        .get("medium")
        .and_then(|v| v.as_str())
        .unwrap_or("InPerson")
        .to_string();

    let location = json
        .get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let their_location = json
        .get("theirLocation")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let topics: Vec<String> = json
        .get("topics")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let note = json
        .get("note")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let date = json
        .get("date")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if person_names.is_empty() {
        return Err("LLM did not extract any person names".into());
    }
    if topics.is_empty() {
        return Err("LLM did not extract any topics".into());
    }

    Ok(ParsedInteraction {
        person_names,
        medium,
        location,
        their_location,
        topics,
        note,
        date,
    })
}
