use std::{
    fs::File,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{anyhow, Context};
use clap::Parser;
use termion::color;

#[derive(Parser)]
pub struct Args {
    #[arg(
        value_name = "FILE",
        short,
        long,
        default_value = None,
        help = "Map solution stdin to file"
    )]
    input_file: Option<PathBuf>,

    #[arg(
        value_name = "FILE",
        short,
        long,
        default_value = None,
        help = "Read solution output from file",
    )]
    output_file: Option<PathBuf>,

    #[arg(id = "answer", help = "Path to the file with correct output")]
    answer_file: PathBuf,

    #[arg(id = "solution", help = "Command to run the solution")]
    solution_command: String,
}

#[derive(Debug)]
enum LineCountCheckResult {
    Correct,
    Incorrect { expected: usize, actual: usize },
}

impl LineCountCheckResult {
    pub fn is_correct(&self) -> bool {
        match self {
            Self::Correct => true,
            Self::Incorrect { .. } => false,
        }
    }
}

enum CheckResult {
    Correct,
    Incorrect(String),
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let solution_stdin = args
        .input_file
        .as_deref()
        .map_or(Ok(Stdio::null()), create_solution_input_stream)?;

    let correct_answer = std::fs::read_to_string(args.answer_file.as_path())
        .with_context(|| {
            format!(
                "could not read correct answer from `{}`",
                args.answer_file.display()
            )
        })?;
    let trimmed_correct_answer = correct_answer.trim();

    let solution_output = create_solution_command(&args.solution_command)
        .stdin(solution_stdin)
        .output()
        .with_context(|| "could not read output of the solution".to_string())?;

    if !solution_output.status.success() {
        let solution_stderr = std::str::from_utf8(&solution_output.stderr)
            .with_context(|| "failed to convert solution stderr to UTF-8")?;
        return Err(anyhow!("Stderr:\n{}", solution_stderr)).with_context(
            || "solution terminated with a non-zero exit code".to_string(),
        );
    }

    let _solution_output_from_file: String;
    let trimmed_actual_answer = if let Some(file) = args.output_file.as_deref()
    {
        _solution_output_from_file = std::fs::read_to_string(file)
            .with_context(|| {
                format!(
                    "couldn't read solution output from `{}`",
                    file.display()
                )
            })?;
        _solution_output_from_file.trim()
    } else {
        std::str::from_utf8(&solution_output.stdout)
            .with_context(|| "failed to convert solution output to UTF-8")?
    };

    let line_count_check_result =
        check_line_count(trimmed_correct_answer, trimmed_actual_answer);
    if let LineCountCheckResult::Incorrect { expected, actual } =
        line_count_check_result
    {
        println!(
            "{}Number of lines differs:{} expected {expected}, got {actual}",
            color::Fg(color::Red),
            color::Fg(color::Reset),
        );
    }

    if line_count_check_result.is_correct() {
        match check_lines(trimmed_correct_answer, trimmed_actual_answer) {
            CheckResult::Correct => {
                println!("The answer is correct",)
            }
            CheckResult::Incorrect(message) => {
                println!("{}", message);
            }
        }
    }

    Ok(())
}

fn create_solution_input_stream(path: &Path) -> anyhow::Result<Stdio> {
    File::open(path).map(Stdio::from).with_context(|| {
        format!("could not read test input from `{}`", path.display())
    })
}

fn create_solution_command(command: &str) -> std::process::Command {
    let mut solution_splitted = command.split_whitespace();
    let mut solution_command = Command::new(solution_splitted.next().unwrap());
    let solution_args = solution_splitted;
    solution_command.args(solution_args);

    solution_command
}

fn trim_filter_non_empty(mut line: &str) -> Option<&str> {
    line = line.trim();
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
}

fn check_line_count(
    correct_answer: &str,
    actual_answer: &str,
) -> LineCountCheckResult {
    let correct_line_count = correct_answer
        .lines()
        .filter_map(trim_filter_non_empty)
        .count();

    let actual_line_count = actual_answer
        .lines()
        .filter_map(trim_filter_non_empty)
        .count();

    if correct_line_count != actual_line_count {
        LineCountCheckResult::Incorrect {
            expected: correct_line_count,
            actual: actual_line_count,
        }
    } else {
        LineCountCheckResult::Correct
    }
}

fn check_lines(correct_answer: &str, actual_answer: &str) -> CheckResult {
    let mut message = String::new();
    let mut correct = true;
    let mut correct_lines =
        correct_answer.lines().filter_map(trim_filter_non_empty);

    for (i, cur_line) in actual_answer
        .lines()
        .filter_map(trim_filter_non_empty)
        .enumerate()
    {
        let cur_correct_line = correct_lines.next().unwrap();

        if i + 1 < 10 {
            message.push_str("  ");
        } else if i + 1 < 100 {
            message.push(' ');
        }
        message.push_str(&format!("{} ", i + 1));

        if cur_line == cur_correct_line {
            message.push_str(&format!(
                "{} {} {}\n",
                color::Bg(color::Green),
                cur_line,
                color::Bg(color::Reset)
            ));
        } else {
            correct = false;
            message.push_str(&format!(
                "{} {} {} => expected {} {} {}\n",
                color::Bg(color::Red),
                cur_line,
                color::Bg(color::Reset),
                color::Bg(color::Green),
                cur_correct_line,
                color::Bg(color::Reset)
            ));
        }
    }

    if correct {
        CheckResult::Correct
    } else {
        // pop the last newline character
        message.pop();
        CheckResult::Incorrect(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl CheckResult {
        pub fn is_correct(&self) -> bool {
            match self {
                Self::Correct => true,
                Self::Incorrect { .. } => false,
            }
        }
    }

    #[test]
    fn line_count() {
        let str1 = "1\n2";
        let str2 = "1\n5";
        let str3 = "1\n2\n3";

        assert!(check_line_count(str1, str2).is_correct());
        assert!(!check_line_count(str1, str3).is_correct());
    }

    #[test]
    fn lines() {
        let str1 = "1\n2";
        let str2 = "1\n55";

        assert!(check_lines(str1, str1).is_correct());
        assert!(!check_lines(str1, str2).is_correct());
    }

    #[test]
    fn extra_whitespace() {
        let actual = "\t1 \n  2  \n\n\n";
        let correct = "1\n2";
        assert!(
            check_line_count(correct, actual).is_correct()
                && check_lines(correct, actual).is_correct()
        );
        let actual = "\t1 \n  2 3  \n\n\n";
        assert!(
            check_line_count(correct, actual).is_correct()
                && !check_lines(correct, actual).is_correct()
        );
        let actual = "\t1 \n  2\n3  \n\n\n";
        assert!(!check_line_count(correct, actual).is_correct());

        let correct = "1\n\n\n2";
        let actual = "1\n2";
        assert!(
            check_line_count(correct, actual).is_correct()
                && check_lines(correct, actual).is_correct()
        );
    }
}
