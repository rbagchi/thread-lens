use clap::Parser;
use thread_lens::analyzer::find_chronically_blocked_threads;

mod cli;
mod io;
mod output;

use cli::{Args, Commands, OutputFormat};
use io::read_dumps_from_directory;
use output::{print_json_view, print_text_view, print_yaml_view};

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

fn handle_view(path: String, output: OutputFormat) -> std::io::Result<()> {
    let content = std::fs::read_to_string(path)?;
    match thread_lens::parser::parse_jstack_output(&content) {
        Ok(dump) => match output {
            OutputFormat::Text => print_text_view(&dump),
            OutputFormat::Json => print_json_view(&dump),
            OutputFormat::Yaml => print_yaml_view(&dump),
        },
        Err(e) => eprintln!("Error parsing file: {}", e),
    }
    Ok(())
}