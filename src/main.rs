use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use trlt::cli::{Cli, Command, ConfigAction};
use trlt::config::Config;
use trlt::input::resolve_input;
use trlt::translate::{self, TranslateOptions};
use trlt::wizard;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            wizard::run_wizard()?;
            Ok(())
        }
        Command::Translate {
            input,
            output,
            from,
            to,
            copy,
            provider,
            model,
        } => {
            cmd_translate(TranslateArgs {
                input,
                output,
                from,
                to,
                copy,
                provider,
                model,
                verbose: cli.verbose,
            })
            .await
        }
        Command::Config { action } => cmd_config(action),
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut <Cli as clap::CommandFactory>::command(),
                "trlt",
                &mut io::stdout(),
            );
            Ok(())
        }
    }
}

struct TranslateArgs {
    input: Option<String>,
    output: Option<String>,
    from: Option<String>,
    to: Option<String>,
    copy: bool,
    provider: Option<String>,
    model: Option<String>,
    verbose: bool,
}

async fn cmd_translate(args: TranslateArgs) -> Result<()> {
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) if wizard::can_run_interactive() => {
            eprintln!("No configuration found. Let's set you up first!\n");
            wizard::run_wizard()?
        }
        Err(e) => return Err(e),
    };

    let text = resolve_input(args.input)?;
    let target = args.to.unwrap_or_else(|| config.default_target.clone());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .expect("invalid template"),
    );
    spinner.set_message("Translating...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let result = translate::translate(
        &config,
        TranslateOptions {
            text,
            from: args.from,
            to: target,
            provider_override: args.provider,
            model_override: args.model,
        },
    )
    .await?;

    spinner.finish_and_clear();

    if let Some(output_path) = &args.output {
        std::fs::write(Path::new(output_path), &result)
            .with_context(|| format!("failed to write to {output_path}"))?;
        if args.verbose {
            eprintln!("Written to {output_path}");
        }
    } else {
        println!("{result}");
    }

    if args.copy {
        match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(&result)) {
            Ok(()) => {
                if args.verbose {
                    eprintln!("Copied to clipboard.");
                }
            }
            Err(e) => eprintln!("Failed to copy to clipboard: {e}"),
        }
    }

    Ok(())
}

fn cmd_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let config = Config::load()?;
            let toml_str = config.to_toml_pretty()?;
            print!("{toml_str}");
        }
        ConfigAction::Set { key, value } => {
            let mut config = Config::load().unwrap_or_default();
            config.set_value(&key, &value)?;
            config.save()?;
            println!("Set {key} = {value}");
        }
    }
    Ok(())
}
