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

    #[arg(short, long, help = "Print full output of the solution")]
    print_output: bool,

    #[arg(id = "answer", help = "Path to the file with correct output")]
    answer_file: PathBuf,

    #[arg(id = "solution", help = "Command to run the solution")]
    solution_command: String,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    // open the input file as the stdio for the solution
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

    if args.print_output {
        println!("Solution output:\n{}", trimmed_actual_answer);
    }

    let is_correct =
        check_line_count(trimmed_correct_answer, trimmed_actual_answer)
            && check_lines(trimmed_correct_answer, trimmed_actual_answer);
    if is_correct {
        println!(
            "{}Tests passed{}",
            color::Fg(color::Green),
            color::Fg(color::Reset)
        );
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

fn check_line_count(correct_answer: &str, actual_answer: &str) -> bool {
    let correct_line_count = correct_answer
        .lines()
        .filter_map(trim_filter_non_empty)
        .count();

    let actual_line_count = actual_answer
        .lines()
        .filter_map(trim_filter_non_empty)
        .count();

    if correct_line_count != actual_line_count {
        println!(
                "{}Number of lines differs:{} expected {correct_line_count}, got {actual_line_count}",
                color::Fg(color::Red),
                color::Fg(color::Reset),
            );
        false
    } else {
        true
    }
}

fn check_lines(correct_answer: &str, actual_answer: &str) -> bool {
    let mut res = true;
    let mut correct_lines =
        correct_answer.lines().filter_map(trim_filter_non_empty);

    for (i, cur_line) in actual_answer
        .lines()
        .filter_map(trim_filter_non_empty)
        .enumerate()
    {
        let cur_correct_line = correct_lines.next().unwrap();

        if cur_line != cur_correct_line {
            println!(
                    "{}Line {} differs:{} expected {cur_correct_line}, got {cur_line}",
                    color::Fg(color::Red),
                    i + 1,
                    color::Fg(color::Reset),
                );
            res = false;
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_count() {
        let str1 = "1\n2";
        let str2 = "1\n5";
        let str3 = "1\n2\n3";

        assert!(check_line_count(str1, str2));
        assert!(!check_line_count(str1, str3));
    }

    #[test]
    fn lines() {
        let str1 = "1\n2";
        let str2 = "1\n55";

        assert!(check_lines(str1, str1));
        assert!(!check_lines(str1, str2));
    }

    #[test]
    fn extra_whitespace() {
        let actual = "\t1 \n  2  \n\n\n";
        let correct = "1\n2";
        assert!(
            check_line_count(correct, actual) && check_lines(correct, actual)
        );
        let actual = "\t1 \n  2 3  \n\n\n";
        assert!(
            check_line_count(correct, actual) && !check_lines(correct, actual)
        );
        let actual = "\t1 \n  2\n3  \n\n\n";
        assert!(!check_line_count(correct, actual));

        let correct = "1\n\n\n2";
        let actual = "1\n2";
        assert!(
            check_line_count(correct, actual) && check_lines(correct, actual)
        );
    }
}
