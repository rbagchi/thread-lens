use crate::models::{ThreadDump, NormalizedThread, CategorizedFrame, ThreadCategory};
use crate::analyzer::{categorize_frame, determine_thread_category};
use chrono::Utc;

pub fn parse_jstack_output_openjdk(output: &str) -> Result<ThreadDump, String> {
    let mut threads = Vec::new();
    let mut current_thread: Option<NormalizedThread> = None;
    let mut current_state_line: Option<String> = None;

    for line in output.lines() {
        if line.starts_with('\"') && line.contains("nid=") {
            // Finalize the previous thread
            if let Some(mut thread) = current_thread.take() {
                if let Some(state_line) = current_state_line.take() {
                    thread.state = parse_thread_state(&state_line);
                }
                thread.category = determine_thread_category(&thread.frames);
                threads.push(thread);
            }

            let name = line.split('\"').nth(1).unwrap_or("").to_string();
            current_thread = Some(NormalizedThread {
                name,
                state: "UNKNOWN".to_string(), // Will be parsed from the state line
                category: ThreadCategory::Unknown,
                frames: Vec::new(),
            });

        } else if line.trim().starts_with("java.lang.Thread.State:") {
            current_state_line = Some(line.trim().to_string());

        } else if let Some(ref mut thread) = current_thread {
            if line.trim().starts_with("at ") {
                let frame_line = line.trim().to_string();
                let category = categorize_frame(&frame_line);
                thread.frames.push(CategorizedFrame {
                    line: frame_line,
                    category,
                });
            }
        }
    }

    // Finalize the last thread
    if let Some(mut thread) = current_thread.take() {
        if let Some(state_line) = current_state_line.take() {
            thread.state = parse_thread_state(&state_line);
        }
        thread.category = determine_thread_category(&thread.frames);
        threads.push(thread);
    }

    Ok(ThreadDump {
        jvm_version: "OpenJDK (Unknown Version)".to_string(), // Placeholder
        timestamp: Utc::now(), // Placeholder
        threads,
    })
}

fn parse_thread_state(state_line: &str) -> String {
    let parts: Vec<&str> = state_line.split_whitespace().collect();
    if parts.len() > 1 {
        let state_str = parts[1];
        match state_str {
            "RUNNABLE" => "RUNNABLE".to_string(),
            "BLOCKED" => "BLOCKED".to_string(),
            "WAITING" => {
                if state_line.contains("(parking)") || state_line.contains("(on object monitor)") {
                    "BLOCKED".to_string()
                } else {
                    "WAITING".to_string()
                }
            },
            "TIMED_WAITING" => "TIMED_WAITING".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    } else {
        "UNKNOWN".to_string()
    }
}
