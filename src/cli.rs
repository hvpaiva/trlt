use clap::{Parser, Subcommand};

/// A multi-provider CLI translator powered by LLMs.
///
/// Translate text from one language to another using OpenAI, Anthropic, or Ollama.
/// Input can be provided as text, a file path, or piped via stdin.
#[derive(Debug, Parser)]
#[command(name = "trlt", version, author, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Increase verbosity
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize the trlt configuration file.
    Init {
        /// The default provider to use.
        #[arg(short, long, default_value = "openai")]
        provider: String,
    },
    /// Translate text, file, or stdin.
    Translate {
        /// Input text or file path. Reads stdin if omitted or "-".
        input: Option<String>,

        /// Output file path. Writes to stdout if omitted.
        #[arg(short, long)]
        output: Option<String>,

        /// Source language. Auto-detected if omitted.
        #[arg(short, long)]
        from: Option<String>,

        /// Target language.
        #[arg(short, long)]
        to: Option<String>,

        /// Copy result to clipboard.
        #[arg(short, long)]
        copy: bool,

        /// Override the configured provider.
        #[arg(short, long)]
        provider: Option<String>,

        /// Override the configured model.
        #[arg(short, long)]
        model: Option<String>,
    },
    /// Manage configuration.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for.
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration.
    Show,
    /// Set a configuration value (e.g., "openai.api_key" "sk-...").
    Set {
        /// Configuration key (e.g., "provider", "default_target", "openai.api_key").
        key: String,
        /// Value to set.
        value: String,
    },
}
