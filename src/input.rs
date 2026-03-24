use std::io::{self, IsTerminal, Read};
use std::path::Path;

use anyhow::{Context, Result};

pub fn resolve_input(input: Option<String>) -> Result<String> {
    match input {
        Some(text) if text == "-" => read_stdin(),
        Some(text) => {
            let path = Path::new(&text);
            if path.is_file() {
                std::fs::read_to_string(path)
                    .with_context(|| format!("failed to read {}", path.display()))
            } else {
                Ok(text)
            }
        }
        None => {
            if io::stdin().is_terminal() {
                anyhow::bail!("No input provided. Pass text, a file path, or pipe via stdin.");
            }
            read_stdin()
        }
    }
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .context("failed to read stdin")?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn resolve_plain_text() {
        let result = resolve_input(Some("Hello world".into())).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn resolve_file_input() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = std::fs::File::create(&file_path).unwrap();
        write!(file, "file content here").unwrap();

        let result = resolve_input(Some(file_path.to_str().unwrap().into())).unwrap();
        assert_eq!(result, "file content here");
    }

    #[test]
    fn resolve_nonexistent_file_treated_as_text() {
        let result = resolve_input(Some("/nonexistent/path/to/file.txt".into())).unwrap();
        assert_eq!(result, "/nonexistent/path/to/file.txt");
    }

    // Note: resolve_input(None) behavior depends on whether stdin is a terminal.
    // In cargo test, stdin is not a terminal, so it would try to read stdin.
    // The "No input provided" error path is tested via integration tests (cli.rs)
    // where the binary is invoked directly with a TTY.

    #[test]
    fn resolve_multiline_text() {
        let result = resolve_input(Some("line1\nline2\nline3".into())).unwrap();
        assert_eq!(result, "line1\nline2\nline3");
    }

    #[test]
    fn resolve_empty_string() {
        let result = resolve_input(Some("".into())).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn resolve_text_with_special_chars() {
        let result = resolve_input(Some("Hello! @#$% \"world\" 'test'".into())).unwrap();
        assert_eq!(result, "Hello! @#$% \"world\" 'test'");
    }

    #[test]
    fn resolve_unicode_text() {
        let result = resolve_input(Some("Olá mundo! 你好世界 🌍".into())).unwrap();
        assert_eq!(result, "Olá mundo! 你好世界 🌍");
    }
}
