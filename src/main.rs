use std::{
    fs,
    io::{self, Read},
    path::Path,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// The translator CLI (trlt) is a command-line tool to translate text using the OpenAI API.
///
/// It uses the Open AI models to translate text from one language to another.
/// The input can be provided as a file path or a string, and the output can be written to a file or to stdout.
///
/// If no input is provided, it will read from stdin. If no output is provided, it will write to stdout.
///
/// The language to translate from can be auto-detected or specified using the `-f|--from` option.
#[derive(Debug, Parser)]
#[command(name = "trlt", version, author, about, long_about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize the trlt CLI by creating a configuration file in $HOME/.config/trlt.toml.
    Init {
        /// The OpenAI API key. If not provided, it will be read from the environment variable `OPENAI_API_KEY`.
        #[arg(short, long, env = "OPENAI_API_KEY")]
        api_key: Option<String>,
        /// The language model to use. If not provided, it will use the default language model for translation.
        #[arg(short, long, default_value = "gpt-4o-mini")]
        model: String,
    },
    /// Translate text, file or stdin using the OpenAI API.
    Translate {
        /// The input to be translated. If not provided or is "-", read from stdin. This can be a file path or a string.
        input: String,
        /// The output file path. If not provided, write to stdout.
        output: Option<String>,
        /// The language to translate from. If not provided, it will be auto-detected.
        #[arg(short, long)]
        from: Option<String>,
        /// The language to translate to.
        #[arg(short, long, default_value = "en")]
        to: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    api_key: String,
    model: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { api_key, model } => {
            let mut api_key_value = "".to_string();
            if api_key.is_none() {
                println!("Provide the OpenAI API key: ");
                io::stdin()
                    .read_line(&mut api_key_value)
                    .expect("Failed to read from stdin");
            } else {
                api_key_value = api_key.unwrap();
            }

            let config = Config {
                api_key: api_key_value.trim().to_string(),
                model,
            };
            init(&config).unwrap();
        }
        Command::Translate {
            input,
            output,
            from,
            to,
        } => {
            let input_content = if input == "-" {
                let mut buffer = String::new();
                io::stdin()
                    .read_to_string(&mut buffer)
                    .expect("Failed to read from stdin");
                buffer
            } else if Path::new(&input).is_file() {
                fs::read_to_string(&input).expect("Failed to read input file")
            } else {
                input.clone()
            };
            translate(&input_content, &output, &from, &to)
                .await
                .unwrap();
        }
    }
}

fn init(config: &Config) -> Result<()> {
    let mut api_key = config.api_key.clone();
    let model = config.model.clone();
    if api_key.is_empty() {
        println!("Provide the OpenAI API key: ");
        io::stdin().read_to_string(&mut api_key)?;
    }
    write_config(&Config { api_key, model })?;
    println!("Config file created successfully in $HOME/.config/trlt.toml");
    Ok(())
}

async fn translate(
    input: &str,
    output: &Option<String>,
    from: &Option<String>,
    to: &str,
) -> Result<()> {
    let config = read_config().expect("Failed to read config file. Please run `trlt init --help` to help you create a config file.");
    let client = reqwest::Client::new();

    let prompt = if let Some(from_lang) = from {
        format!("Translate this from {} to {}: {}", from_lang, to, input)
    } else {
        format!("Translate this to {}: {}", to, input)
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [{
                "role": "system",
                "content": "You are a translator that only gives the translated text."
            }, {
                "role": "user",
                "content": prompt
            }]
        }))
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;

    if response_json["error"]["message"].as_str().is_some() {
        return Err(anyhow::anyhow!(
            "Failed to translate text: {}",
            response_json["error"]["message"]
        ));
    }

    let response_text = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap()
        .to_string();

    if response_text.is_empty() {
        return Err(anyhow::anyhow!(
            "Failed to translate text: Empty response from API"
        ));
    }

    if let Some(output_path) = output {
        let path = Path::new(output_path);
        fs::write(path, response_text)?;
    } else {
        println!("{}", response_text);
    }

    Ok(())
}

fn read_config() -> Result<Config> {
    let config_path = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join("trlt.toml");
    let config: Config = toml::from_str(&fs::read_to_string(config_path)?)?;
    Ok(config)
}

fn write_config(config: &Config) -> Result<()> {
    let config_path = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join("trlt.toml");
    fs::write(config_path, toml::to_string_pretty(config)?)?;
    Ok(())
}
