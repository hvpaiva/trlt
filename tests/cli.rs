use assert_cmd::Command;
use predicates::prelude::*;

fn trlt() -> Command {
    Command::cargo_bin("trlt").unwrap()
}

/// Write a minimal config file to a temp XDG dir and return the dir.
fn setup_config(provider: &str, api_key: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("trlt");
    std::fs::create_dir_all(&config_dir).unwrap();

    let mut config = format!("provider = \"{provider}\"\ndefault_target = \"en\"\n");

    match provider {
        "openai" => {
            if let Some(key) = api_key {
                config.push_str(&format!(
                    "\n[openai]\napi_key = \"{key}\"\nmodel = \"gpt-4o-mini\"\n"
                ));
            }
        }
        "anthropic" => {
            if let Some(key) = api_key {
                config.push_str(&format!(
                    "\n[anthropic]\napi_key = \"{key}\"\nmodel = \"claude-haiku-4-5-20251001\"\n"
                ));
            }
        }
        "ollama" => {
            config.push_str("\n[ollama]\nmodel = \"llama3\"\nbase_url = \"http://localhost:1\"\n");
        }
        _ => {}
    }

    std::fs::write(config_dir.join("config.toml"), config).unwrap();
    dir
}

// --- Help & version ---

#[test]
fn help_shows_usage() {
    trlt()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("multi-provider CLI translator"))
        .stdout(predicate::str::contains("translate"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn version_shows_version() {
    trlt()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("trlt"));
}

#[test]
fn translate_help_shows_options() {
    trlt()
        .args(["translate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--from"))
        .stdout(predicate::str::contains("--to"))
        .stdout(predicate::str::contains("--copy"))
        .stdout(predicate::str::contains("--provider"))
        .stdout(predicate::str::contains("--model"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn config_help_shows_subcommands() {
    trlt()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("set"));
}

#[test]
fn init_help_shows_setup() {
    trlt()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Set up trlt interactively"));
}

#[test]
fn unknown_subcommand_fails() {
    trlt()
        .arg("foobar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

// --- Completions ---

#[test]
fn completions_bash() {
    trlt()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_trlt"));
}

#[test]
fn completions_zsh() {
    trlt()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("trlt"));
}

#[test]
fn completions_fish() {
    trlt()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("trlt"));
}

// --- Init (interactive wizard) ---

#[test]
fn init_wizard_with_piped_input() {
    let dir = tempfile::tempdir().unwrap();
    // Pipe answers: provider=1 (openai), api_key, model (default), target lang (default)
    trlt()
        .arg("init")
        .env("XDG_CONFIG_HOME", dir.path())
        .write_stdin("1\nsk-test-key\ngpt-4o-mini\nen\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Welcome to trlt"))
        .stderr(predicate::str::contains("Config saved to"));

    let config_path = dir.path().join("trlt").join("config.toml");
    assert!(config_path.exists());

    let contents = std::fs::read_to_string(&config_path).unwrap();
    assert!(contents.contains("provider = \"openai\""));
    assert!(contents.contains("api_key = \"sk-test-key\""));
    // Commented-out sections for other providers
    assert!(contents.contains("# [anthropic]"));
    assert!(contents.contains("# [ollama]"));
}

#[test]
fn init_wizard_anthropic() {
    let dir = tempfile::tempdir().unwrap();
    trlt()
        .arg("init")
        .env("XDG_CONFIG_HOME", dir.path())
        .write_stdin("2\nsk-ant-test\n\npt\n")  // provider, api_key, model (default), target
        .assert()
        .success();

    let contents = std::fs::read_to_string(dir.path().join("trlt").join("config.toml")).unwrap();
    assert!(contents.contains("provider = \"anthropic\""));
    assert!(contents.contains("default_target = \"pt\""));
    assert!(contents.contains("api_key = \"sk-ant-test\""));
    assert!(contents.contains("# [openai]"));
}

#[test]
fn init_wizard_ollama() {
    let dir = tempfile::tempdir().unwrap();
    trlt()
        .arg("init")
        .env("XDG_CONFIG_HOME", dir.path())
        .write_stdin("3\n\n\nes\n")  // provider, model (default), url (default), target
        .assert()
        .success();

    let contents = std::fs::read_to_string(dir.path().join("trlt").join("config.toml")).unwrap();
    assert!(contents.contains("provider = \"ollama\""));
    assert!(contents.contains("default_target = \"es\""));
    assert!(contents.contains("[ollama]"));
    assert!(contents.contains("# [openai]"));
}

// --- Config ---

#[test]
fn config_show() {
    let dir = setup_config("openai", Some("sk-test"));
    trlt()
        .args(["config", "show"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("provider = \"openai\""))
        .stdout(predicate::str::contains("default_target = \"en\""));
}

#[test]
fn config_set_top_level() {
    let dir = setup_config("openai", Some("sk-test"));
    trlt()
        .args(["config", "set", "default_target", "pt"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Set default_target = pt"));

    trlt()
        .args(["config", "show"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("default_target = \"pt\""));
}

#[test]
fn config_set_nested_key() {
    let dir = setup_config("openai", Some("sk-old"));
    trlt()
        .args(["config", "set", "openai.api_key", "sk-new"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success();

    trlt()
        .args(["config", "show"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("api_key = \"sk-new\""));
}

// --- Translate ---

#[test]
fn translate_no_config_no_terminal_fails() {
    let dir = tempfile::tempdir().unwrap();
    trlt()
        .args(["translate", "hello"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No configuration found"));
}

#[test]
fn translate_provider_not_configured() {
    let dir = setup_config("openai", None);
    trlt()
        .args(["translate", "hello"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("OpenAI not configured"));
}

#[test]
fn translate_reads_from_file() {
    let dir = setup_config("openai", Some("sk-fake"));

    let file_path = dir.path().join("input.txt");
    std::fs::write(&file_path, "hello world").unwrap();

    trlt()
        .args(["translate", file_path.to_str().unwrap()])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("OpenAI API error")
                .or(predicate::str::contains("failed to send")),
        );
}
