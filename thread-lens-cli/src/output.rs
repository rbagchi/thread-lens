use thread_lens::models::ThreadDump;
use serde_json::json;

pub fn print_text_view(dump: &ThreadDump) {
    println!("--- Thread Dump Analysis ---");
    println!("JVM Version: {}", dump.jvm_version);
    println!("Timestamp: {}", dump.timestamp);
    println!("Total Threads: {}", dump.threads.len());
    println!("\n--- Threads ---");
    for thread in &dump.threads {
        println!("\n- Name: {}", thread.name);
        println!("  State: {}", thread.state);
        println!("  Category: {:?}", thread.category);
        println!("  Frames:");
        for frame in &thread.frames {
            println!("    [{:?}] {}", frame.category, frame.line);
        }
    }
}

pub fn print_json_view(dump: &ThreadDump) {
    match serde_json::to_string_pretty(dump) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

pub fn print_yaml_view(dump: &ThreadDump) {
    match serde_yaml::to_string(dump) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => eprintln!("Error serializing to YAML: {}", e),
    }
}
