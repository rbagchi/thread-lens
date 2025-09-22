use crate::models::{ThreadDump, NormalizedThread, CategorizedFrame, ThreadCategory};
use crate::analyzer::{categorize_frame, determine_thread_category};
use chrono::Utc;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref IBM_JVM_VERSION_REGEX: Regex = Regex::new(r"Full thread dump IBM Semeru Runtime Open Edition (\d+\.\d+\.\d+\.\d+)").unwrap();
}

pub fn parse_jstack_output_ibm(output: &str) -> Result<ThreadDump, String> {
    let mut threads = Vec::new();
    let mut current_thread: Option<NormalizedThread> = None;
    let mut jvm_version = "IBM J9 (Unknown Version)".to_string(); // Default placeholder

    for line in output.lines() {
        // Attempt to extract JVM version from header lines
        if let Some(captures) = IBM_JVM_VERSION_REGEX.captures(line) {
            if let Some(version_match) = captures.get(1) {
                jvm_version = format!("IBM Semeru Runtime Open Edition {}", version_match.as_str().trim());
            }
        }

        if line.contains("prio=") && line.contains("tid=") {
            // Finalize the previous thread before starting a new one
            if let Some(mut thread) = current_thread.take() {
                thread.category = determine_thread_category(&thread.frames);
                threads.push(thread);
            }

            let name = line.split('\"').nth(1).unwrap_or("").to_string();
            let state = if line.contains(" BLOCKED") || line.contains("state:B") {
                "BLOCKED".to_string()
            } else if line.contains(" RUNNABLE") || line.contains("state:R") {
                "RUNNABLE".to_string()
            } else if line.contains(" WAITING") || line.contains("PARKED") {
                "WAITING".to_string()
            } else if line.contains(" TIMED_WAITING") || line.contains("SLEEPING") {
                "TIMED_WAITING".to_string()
            } else {
                "UNKNOWN".to_string()
            };

            current_thread = Some(NormalizedThread {
                name,
                state,
                category: ThreadCategory::Unknown, // Will be determined after parsing frames
                frames: Vec::new(),
            });

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

    // Finalize the last thread in the file
    if let Some(mut thread) = current_thread.take() {
        thread.category = determine_thread_category(&thread.frames);
        threads.push(thread);
    }

    Ok(ThreadDump {
        jvm_version, // Use the extracted JVM version
        timestamp: Utc::now(), // Placeholder
        threads,
    })
}
