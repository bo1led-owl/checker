use std::{
    cmp::max,
    io::Write,
    os::unix::process::ExitStatusExt,
    path::PathBuf,
    process::{Command, Output},
};

use anyhow::{anyhow, Context};
use clap::Parser;
use termion::{color, style};

#[derive(Parser)]
pub struct Args {
    #[arg(id = "TESTS", help = "Path to the file with test suite description")]
    test_suite: PathBuf,

    #[arg(id = "SOLUTION", help = "Command to run the solution")]
    solution_command: String,
}

enum CheckResult {
    Correct,
    Incorrect { message: String },
}

struct Test<'a> {
    input: &'a str,
    answer: &'a str,
}

impl<'a> Test<'a> {
    pub fn new(input: &'a str, answer: &'a str) -> Self {
        Self { input, answer }
    }
}

fn parse_test(s: &str) -> anyhow::Result<Test> {
    let body = match s.strip_prefix("[input]\n") {
        Some(stripped) => stripped,
        None => return Err(anyhow!("`[input]` header is not present")),
    };

    let (input, answer) = match body.split_once("[answer]\n") {
        Some((i, a)) => (i.trim(), a.trim()),
        None => return Err(anyhow!("`[answer]` header is not present")),
    };

    Ok(Test::new(input, answer))
}

fn parse_tests(source: &str) -> anyhow::Result<Vec<Test>> {
    source
        .split("[test]\n")
        .filter(|test| !test.is_empty())
        .map(parse_test)
        .collect()
}

fn run_test<'a>(test: Test<'a>, command: &str) -> anyhow::Result<()> {
    let mut child = create_solution_command(command)
        .spawn()
        .with_context(|| "Couldn't spawn child process")?;

    child
        .stdin
        .take()
        .with_context(|| "Couldn't open child stdin")?
        .write_all(test.input.as_bytes())
        .with_context(|| "Couldn't write to child stdin")?;

    let output = child
        .wait_with_output()
        .with_context(|| "could not read output of the solution")?;
    report_if_solution_terminated_correctly(&output)?;

    let actual_answer = std::str::from_utf8(&output.stdout)
        .with_context(|| "failed to convert solution stdout to UTF-8")?
        .trim();

    match check_lines(test.answer, actual_answer) {
        CheckResult::Correct => {
            println!(
                "{}{}Passed{}{}",
                style::Bold,
                color::Fg(color::Green),
                style::Reset,
                color::Fg(color::Reset)
            )
        }
        CheckResult::Incorrect { message } => {
            println!(
                "{}{}Wrong answer{}{}",
                style::Bold,
                color::Fg(color::Red),
                style::Reset,
                color::Fg(color::Reset)
            );
            print!("{message}");
        }
    }

    Ok(())
}

pub fn run(args: Args) -> anyhow::Result<()> {
    if args.solution_command.is_empty() {
        return Err(anyhow!("Empty solution command provided"));
    }

    let tests_source =
        std::fs::read_to_string(&args.test_suite).with_context(|| {
            format!(
                "Couldn't read test suite description from `{}`",
                args.test_suite.display()
            )
        })?;
    let tests = parse_tests(&tests_source)?;

    for (i, test) in tests.into_iter().enumerate() {
        print!("{}Test {}: {}", style::Bold, i + 1, style::Reset);
        match run_test(test, args.solution_command.trim()) {
            Ok(_) => {}
            Err(err) => {
                println!(
                    "{}{}Error occured{}",
                    style::Bold,
                    color::Fg(color::Red),
                    style::Reset,
                );
                println!("{err}:");
                err.chain().skip(1).for_each(|cause| println!("{}", cause));
            }
        }
    }

    Ok(())
}

fn create_solution_command(command: &str) -> Command {
    let mut solution_splitted = command.split_whitespace();
    let mut solution_command = Command::new(solution_splitted.next().unwrap());
    let solution_args = solution_splitted;
    solution_command.args(solution_args);

    solution_command
}

fn report_if_solution_terminated_correctly(
    output: &Output,
) -> Result<(), anyhow::Error> {
    if output.status.success() {
        return Ok(());
    }

    if let Some(libc::SIGSEGV) = output.status.signal() {
        Err(anyhow!("Segmentation fault"))
    } else {
        Err(anyhow!(
            "{}",
            std::str::from_utf8(&output.stderr).with_context(|| {
                "failed to convert solution stderr to UTF-8"
            })?
        ))
    }
    .with_context(|| {
        "Solution terminated with a non-zero exit code".to_string()
    })
}

fn trim_filter_non_empty(mut line: &str) -> Option<&str> {
    line = line.trim();
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
}

fn get_integer_length(mut n: usize) -> usize {
    let mut result = 0;
    while n > 0 {
        n /= 10;
        result += 1;
    }
    return result;
}

fn check_lines(correct_answer: &str, actual_answer: &str) -> CheckResult {
    let mut message = String::new();
    let mut correct = true;
    let mut correct_lines =
        correct_answer.lines().filter_map(trim_filter_non_empty);
    let mut actual_lines =
        actual_answer.lines().filter_map(trim_filter_non_empty);

    let max_line_count =
        max(correct_lines.clone().count(), actual_lines.clone().count());

    let max_line_number_len = get_integer_length(max_line_count);

    for i in 1..=max_line_count {
        let cur_line = actual_lines.next().unwrap_or("");
        let cur_correct_line = correct_lines.next().unwrap_or("");

        let mut cur_line_number_formatted = String::new();

        // offset line numbers to appear evenly
        for _ in 0..max_line_number_len {
            cur_line_number_formatted.push(' ');
        }

        cur_line_number_formatted.push_str(&format!("{i} "));

        if cur_line != cur_correct_line {
            correct = false;
            message.push_str(&format!(
                "{} {} {cur_line} {} => expected {} {cur_correct_line} {}\n",
                cur_line_number_formatted,
                color::Bg(color::Red),
                color::Bg(color::Reset),
                color::Bg(color::Green),
                color::Bg(color::Reset)
            ));
        }
    }

    if correct {
        CheckResult::Correct
    } else {
        CheckResult::Incorrect { message }
    }
}
