use crate::models::{FrameCategory, ThreadCategory, CategorizedFrame, ThreadDump, NormalizedThread};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    static ref JVM_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"^\s*at java\.").unwrap(),
        Regex::new(r"^\s*at sun\.").unwrap(),
        Regex::new(r"^\s*at jdk\.").unwrap(),
    ];
    static ref FRAMEWORK_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"^\s*at (?:app//)?org\.eclipse\.jetty\.").unwrap(),
        Regex::new(r"^\s*at (?:app//)?spark\.").unwrap(),
    ];
}

pub fn categorize_frame(frame_line: &str) -> FrameCategory {
    for pattern in JVM_PATTERNS.iter() {
        if pattern.is_match(frame_line) {
            return FrameCategory::Jvm;
        }
    }
    for pattern in FRAMEWORK_PATTERNS.iter() {
        if pattern.is_match(frame_line) {
            return FrameCategory::Framework;
        }
    }
    FrameCategory::Application
}

pub fn determine_thread_category(frames: &[CategorizedFrame]) -> ThreadCategory {
    let mut has_application_frame = false;
    let mut has_framework_frame = false;

    for frame in frames {
        match frame.category {
            FrameCategory::Application => {
                has_application_frame = true;
                break;
            }
            FrameCategory::Framework => {
                has_framework_frame = true;
            }
            FrameCategory::Jvm => {}
        }
    }

    if has_application_frame {
        ThreadCategory::Application
    } else if has_framework_frame {
        ThreadCategory::Framework
    } else {
        ThreadCategory::Jvm
    }
}

pub fn find_chronically_blocked_threads(dumps: &[ThreadDump]) -> HashMap<String, (NormalizedThread, usize)> {
    let mut blocked_counts: HashMap<String, usize> = HashMap::new();
    let mut latest_threads: HashMap<String, NormalizedThread> = HashMap::new();

    for dump in dumps {
        for thread in &dump.threads {
            if thread.state == "BLOCKED" && thread.category == ThreadCategory::Application {
                *blocked_counts.entry(thread.name.clone()).or_insert(0) += 1;
            }
            latest_threads.insert(thread.name.clone(), thread.clone());
        }
    }

    let mut chronically_blocked = HashMap::new();
    for (name, count) in blocked_counts {
        if count > 1 {
            if let Some(thread) = latest_threads.get(&name) {
                chronically_blocked.insert(name, (thread.clone(), count));
            }
        }
    }

    chronically_blocked
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_find_chronically_blocked_threads() {
        let mut dumps = Vec::new();

        // Dump 1
        let thread1 = NormalizedThread {
            name: "Thread-1".to_string(),
            state: "BLOCKED".to_string(),
            category: ThreadCategory::Application,
            frames: vec![],
        };
        let dump1 = ThreadDump {
            jvm_version: "1.8.0".to_string(),
            timestamp: Utc::now(),
            threads: vec![thread1],
        };
        dumps.push(dump1);

        // Dump 2
        let thread2 = NormalizedThread {
            name: "Thread-1".to_string(),
            state: "BLOCKED".to_string(),
            category: ThreadCategory::Application,
            frames: vec![],
        };
        let dump2 = ThreadDump {
            jvm_version: "1.8.0".to_string(),
            timestamp: Utc::now(),
            threads: vec![thread2],
        };
        dumps.push(dump2);

        let chronically_blocked = find_chronically_blocked_threads(&dumps);
        assert_eq!(chronically_blocked.len(), 1);
        assert!(chronically_blocked.contains_key("Thread-1"));
        assert_eq!(chronically_blocked.get("Thread-1").unwrap().1, 2);
    }
}
