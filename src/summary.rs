use crate::tracking;
use crate::utils;
use crate::utils::truncate;
use anyhow::{Context, Result};
use std::process::Stdio;

/// Run a command and provide a heuristic summary
pub fn run(command: &str, verbose: u8) -> Result<()> {
    let timer = tracking::TimedExecution::start();

    if verbose > 0 {
        eprintln!("Running and summarizing: {}", command);
    }

    let output = utils::shell_command(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to execute command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw = format!("{}\n{}", stdout, stderr);

    let summary = summarize_output(&raw, command, output.status.success());
    println!("{}", summary);
    timer.track(command, "rtk summary", &raw, &summary);
    Ok(())
}

fn summarize_output(output: &str, command: &str, success: bool) -> String {
    let mut result = Vec::new();

    // Status
    let status_icon = if success { "[ok]" } else { "[FAIL]" };
    result.push(format!(
        "{} Command: {}",
        status_icon,
        truncate(command, 60)
    ));
    result.push(format!("   {} lines of output", output.lines().count()));
    result.push(String::new());

    // Detect type of output and summarize accordingly
    let output_type = detect_output_type(output, command);

    match output_type {
        OutputType::TestResults => summarize_tests(output, &mut result),
        OutputType::BuildOutput => summarize_build(output, &mut result),
        OutputType::LogOutput => summarize_logs_quick(output, &mut result),
        OutputType::ListOutput => summarize_list(output, &mut result),
        OutputType::JsonOutput => summarize_json(output, &mut result),
        OutputType::Generic => summarize_generic(output, &mut result),
    }

    result.join("\n")
}

#[derive(Debug)]
enum OutputType {
    TestResults,
    BuildOutput,
    LogOutput,
    ListOutput,
    JsonOutput,
    Generic,
}

fn detect_output_type(output: &str, command: &str) -> OutputType {
    if contains_ascii_insensitive(command, "test")
        || contains_ascii_insensitive(output, "passed") && contains_ascii_insensitive(output, "failed")
    {
        OutputType::TestResults
    } else if contains_ascii_insensitive(command, "build")
        || contains_ascii_insensitive(command, "compile")
        || contains_ascii_insensitive(output, "compiling")
    {
        OutputType::BuildOutput
    } else if contains_ascii_insensitive(output, "error:")
        || contains_ascii_insensitive(output, "warn:")
        || contains_ascii_insensitive(output, "[info]")
    {
        OutputType::LogOutput
    } else if output.trim_start().starts_with('{') || output.trim_start().starts_with('[') {
        OutputType::JsonOutput
    } else if output.lines().all(|l| {
        l.len() < 200
            && if l.contains('\t') {
                false
            } else {
                l.split_whitespace().count() < 10
            }
    }) {
        OutputType::ListOutput
    } else {
        OutputType::Generic
    }
}

fn summarize_tests(output: &str, result: &mut Vec<String>) {
    result.push("Test Results:".to_string());

    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut failures = Vec::new();

    for line in output.lines() {
        if contains_ascii_insensitive(line, "passed") || line.contains('✓') || contains_ascii_insensitive(line, "ok") {
            // Try to extract number
            if let Some(n) = extract_number(line, "passed") {
                passed = n;
            } else {
                passed += 1;
            }
        }
        if contains_ascii_insensitive(line, "failed")
            || contains_ascii_insensitive(line, "[x]")
            || contains_ascii_insensitive(line, "fail")
        {
            if let Some(n) = extract_number(line, "failed") {
                failed = n;
            }
            if !contains_ascii_insensitive(line, "0 failed") {
                failures.push(line.to_string());
            }
        }
        if contains_ascii_insensitive(line, "skipped") || contains_ascii_insensitive(line, "ignored") {
            if let Some(n) = extract_number(line, "skipped").or(extract_number(line, "ignored")) {
                skipped = n;
            }
        }
    }

    result.push(format!("   [ok] {} passed", passed));
    if failed > 0 {
        result.push(format!("   [FAIL] {} failed", failed));
    }
    if skipped > 0 {
        result.push(format!("   skip {} skipped", skipped));
    }

    if !failures.is_empty() {
        result.push(String::new());
        result.push("   Failures:".to_string());
        for f in failures.iter().take(5) {
            result.push(format!("   • {}", truncate(f, 70)));
        }
    }
}

fn summarize_build(output: &str, result: &mut Vec<String>) {
    result.push("Build Summary:".to_string());

    let mut errors = 0;
    let mut warnings = 0;
    let mut compiled = 0;
    let mut error_msgs = Vec::new();

    for line in output.lines() {
        if contains_ascii_insensitive(line, "error") && !contains_ascii_insensitive(line, "0 error") {
            errors += 1;
            if error_msgs.len() < 5 {
                error_msgs.push(line.to_string());
            }
        }
        if contains_ascii_insensitive(line, "warning") && !contains_ascii_insensitive(line, "0 warning") {
            warnings += 1;
        }
        if contains_ascii_insensitive(line, "compiling") || contains_ascii_insensitive(line, "compiled") {
            compiled += 1;
        }
    }

    if compiled > 0 {
        result.push(format!("   {} crates/files compiled", compiled));
    }
    if errors > 0 {
        result.push(format!("   [error] {} errors", errors));
    }
    if warnings > 0 {
        result.push(format!("   [warn] {} warnings", warnings));
    }
    if errors == 0 && warnings == 0 {
        result.push("   [ok] Build successful".to_string());
    }

    if !error_msgs.is_empty() {
        result.push(String::new());
        result.push("   Errors:".to_string());
        for e in &error_msgs {
            result.push(format!("   • {}", truncate(e, 70)));
        }
    }
}

fn summarize_logs_quick(output: &str, result: &mut Vec<String>) {
    result.push("Log Summary:".to_string());

    let mut errors = 0;
    let mut warnings = 0;
    let mut info = 0;

    for line in output.lines() {
        if contains_ascii_insensitive(line, "error") || contains_ascii_insensitive(line, "fatal") {
            errors += 1;
        } else if contains_ascii_insensitive(line, "warn") {
            warnings += 1;
        } else if contains_ascii_insensitive(line, "info") {
            info += 1;
        }
    }

    result.push(format!("   [error] {} errors", errors));
    result.push(format!("   [warn] {} warnings", warnings));
    result.push(format!("   [info] {} info", info));
}

fn summarize_list(output: &str, result: &mut Vec<String>) {
    let lines: Vec<&str> = output.lines().filter(|l| !l.trim().is_empty()).collect();
    result.push(format!("List ({} items):", lines.len()));

    for line in lines.iter().take(10) {
        result.push(format!("   • {}", truncate(line, 70)));
    }
    if lines.len() > 10 {
        result.push(format!("   ... +{} more", lines.len() - 10));
    }
}

fn summarize_json(output: &str, result: &mut Vec<String>) {
    result.push("JSON Output:".to_string());

    // Try to parse and show structure
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(output) {
        match &value {
            serde_json::Value::Array(arr) => {
                result.push(format!("   Array with {} items", arr.len()));
            }
            serde_json::Value::Object(obj) => {
                result.push(format!("   Object with {} keys:", obj.len()));
                for key in obj.keys().take(10) {
                    result.push(format!("   • {}", key));
                }
                if obj.len() > 10 {
                    result.push(format!("   ... +{} more keys", obj.len() - 10));
                }
            }
            _ => {
                result.push(format!("   {}", truncate(&value.to_string(), 100)));
            }
        }
    } else {
        result.push("   (Invalid JSON)".to_string());
    }
}

fn summarize_generic(output: &str, result: &mut Vec<String>) {
    let lines: Vec<&str> = output.lines().collect();

    result.push("Output:".to_string());

    // First few lines
    for line in lines.iter().take(5) {
        if !line.trim().is_empty() {
            result.push(format!("   {}", truncate(line, 75)));
        }
    }

    if lines.len() > 10 {
        result.push("   ...".to_string());
        // Last few lines
        for line in lines.iter().skip(lines.len() - 3) {
            if !line.trim().is_empty() {
                result.push(format!("   {}", truncate(line, 75)));
            }
        }
    }
}

fn contains_ascii_insensitive(text: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let haystack = text.as_bytes();
    let needle = needle.as_bytes();

    haystack
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

fn extract_number(text: &str, after: &str) -> Option<usize> {
    let after = after.as_bytes();
    let bytes = text.as_bytes();

    let mut idx = 0usize;
    while idx < bytes.len() {
        if bytes[idx].is_ascii_digit() {
            let start = idx;
            while idx < bytes.len() && bytes[idx].is_ascii_digit() {
                idx += 1;
            }

            let digits = &text[start..idx];
            let mut probe = idx;
            while probe < bytes.len() && bytes[probe].is_ascii_whitespace() {
                probe += 1;
            }

            if probe + after.len() <= bytes.len()
                && bytes[probe..probe + after.len()].eq_ignore_ascii_case(after)
            {
                return digits.parse().ok();
            }
        } else {
            idx += 1;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_ascii_insensitive() {
        assert!(contains_ascii_insensitive("Build SUCCESS", "success"));
        assert!(contains_ascii_insensitive("[INFO] started", "[info]"));
        assert!(!contains_ascii_insensitive("warning", "fatal"));
    }

    #[test]
    fn test_extract_number_matches_keyword_case_insensitive() {
        assert_eq!(extract_number("15 passed", "passed"), Some(15));
        assert_eq!(extract_number("2 FAILED", "failed"), Some(2));
        assert_eq!(extract_number("7 ignored", "ignored"), Some(7));
    }

    #[test]
    fn test_extract_number_ignores_non_matching_sequences() {
        assert_eq!(extract_number("abc123 xyz", "passed"), None);
        assert_eq!(extract_number("0 warnings", "failed"), None);
    }

    #[test]
    fn test_detect_output_type_without_lowercasing() {
        assert!(matches!(
            detect_output_type("PASSED 10 tests, FAILED 1", "runner"),
            OutputType::TestResults
        ));
        assert!(matches!(
            detect_output_type("[INFO] startup complete", "server"),
            OutputType::LogOutput
        ));
    }
}
