use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}
