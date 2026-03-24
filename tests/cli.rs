use assert_cmd::Command;
use predicates::prelude::*;

fn trlt() -> Command {
    Command::cargo_bin("trlt").unwrap()
}

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

#[test]
fn translate_reads_from_file() {
    let dir = tempfile::tempdir().unwrap();

    // Init config with openai but fake key - will fail at API call, not input resolution
    trlt()
        .arg("init")
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success();

    trlt()
        .args(["config", "set", "openai.api_key", "sk-fake"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success();

    // Create a test file
    let file_path = dir.path().join("input.txt");
    std::fs::write(&file_path, "hello world").unwrap();

    // Translate from file - should fail at API level, proving input was resolved
    trlt()
        .args(["translate", file_path.to_str().unwrap()])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("OpenAI API error").or(predicate::str::contains("API")));
}

#[test]
fn init_creates_config() {
    let dir = tempfile::tempdir().unwrap();
    trlt()
        .args(["init", "--provider", "anthropic"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Config created at"));

    let config_path = dir.path().join("trlt").join("config.toml");
    assert!(config_path.exists());

    let contents = std::fs::read_to_string(&config_path).unwrap();
    assert!(contents.contains("provider = \"anthropic\""));
}

#[test]
fn init_default_provider_is_openai() {
    let dir = tempfile::tempdir().unwrap();
    trlt()
        .arg("init")
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success();

    let config_path = dir.path().join("trlt").join("config.toml");
    let contents = std::fs::read_to_string(&config_path).unwrap();
    assert!(contents.contains("provider = \"openai\""));
}

#[test]
fn config_show_after_init() {
    let dir = tempfile::tempdir().unwrap();
    trlt()
        .arg("init")
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success();

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
    let dir = tempfile::tempdir().unwrap();
    trlt()
        .arg("init")
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success();

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
    let dir = tempfile::tempdir().unwrap();
    trlt()
        .arg("init")
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success();

    trlt()
        .args(["config", "set", "openai.api_key", "sk-test123"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success();

    trlt()
        .args(["config", "show"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("api_key = \"sk-test123\""));
}

#[test]
fn translate_with_no_config_fails() {
    let dir = tempfile::tempdir().unwrap();
    trlt()
        .args(["translate", "hello"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No configuration found"));
}

#[test]
fn translate_stdin_pipe() {
    let dir = tempfile::tempdir().unwrap();
    // Init with openai but no API key set - translate should fail at provider creation
    trlt()
        .arg("init")
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .success();

    trlt()
        .args(["translate", "hello"])
        .env("XDG_CONFIG_HOME", dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("OpenAI not configured"));
}

#[test]
fn unknown_subcommand_fails() {
    trlt()
        .arg("foobar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}
