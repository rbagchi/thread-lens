use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::Path;
use thread_lens::models::ThreadDump;
use thread_lens::parser::parse_jstack_output;
use thread_lens::analyzer::find_chronically_blocked_threads;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze a directory of thread dumps for chronically blocked threads
    Analyze {
        /// Path to a directory containing jstack files
        #[arg(short, long)]
        path: String,
    },
    /// View a single thread dump in a normalized format
    View {
        /// Path to a single jstack file
        #[arg(short, long)]
        path: String,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Text,
    Json,
    Yaml,
}

fn main() -> std::io::Result<()> {
    env_logger::init();
    let args = Args::parse();

    match args.command {
        Commands::Analyze { path } => handle_analyze(path)?,
        Commands::View { path, output } => handle_view(path, output)?,
    }

    Ok(())
}

fn handle_analyze(path: String) -> std::io::Result<()> {
    let dumps = read_dumps_from_directory(path)?;

    println!("--- Analysis Report ---");
    println!("Found {} thread dumps to analyze.", dumps.len());

    let chronically_blocked = find_chronically_blocked_threads(&dumps);

    if chronically_blocked.is_empty() {
        println!("\nNo chronically blocked application threads found.");
    } else {
        println!("\nFound {} chronically blocked application threads:", chronically_blocked.len());
        for (name, (thread, count)) in chronically_blocked {
            println!("  - Thread: '{}' (blocked in {} dumps)", name, count);
            println!("    State: {}", thread.state);
            println!("    Category: {:?}", thread.category);
            println!("    Stack Trace:");
            for frame in thread.frames.iter().take(5) {
                println!("      [{:?}] {}", frame.category, frame.line);
            }
        }
    }

    Ok(())
}

fn read_dumps_from_directory(dir_path: String) -> std::io::Result<Vec<ThreadDump>> {
    let mut dumps = Vec::new();
    let path = Path::new(&dir_path);

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "jstack") {
                let content = fs::read_to_string(&path)?;
                match parse_jstack_output(&content) {
                    Ok(dump) => dumps.push(dump),
                    Err(e) => eprintln!("Error parsing file {}: {}", path.display(), e),
                }
            }
        }
    }
    Ok(dumps)
}

fn handle_view(path: String, output: OutputFormat) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    match parse_jstack_output(&content) {
        Ok(dump) => match output {
            OutputFormat::Text => print_text_view(&dump),
            OutputFormat::Json => print_json_view(&dump),
            OutputFormat::Yaml => print_yaml_view(&dump),
        },
        Err(e) => eprintln!("Error parsing file: {}", e),
    }
    Ok(())
}

fn print_text_view(dump: &ThreadDump) {
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

fn print_json_view(dump: &ThreadDump) {
    match serde_json::to_string_pretty(dump) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

fn print_yaml_view(dump: &ThreadDump) {
    match serde_yaml::to_string(dump) {
        Ok(yaml) => println!("{}", yaml),
        Err(e) => eprintln!("Error serializing to YAML: {}", e),
    }
}
