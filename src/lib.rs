use std::{
    cmp::max,
    io::Write,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use anyhow::{anyhow, Context};
use clap::Parser;
use termion::{color, style};

#[derive(Parser)]
pub struct Args {
    #[arg(
        value_name = "FILE",
        short,
        long,
        default_value = None,
        help = "Read solution output from file",
    )]
    output_file: Option<PathBuf>,

    #[arg(id = "TESTS", help = "Path to the file with test suite description")]
    test_suite: PathBuf,

    #[arg(id = "SOLUTION", help = "Command to run the solution")]
    solution_command: String,
}

enum SolutionAnswer<'a> {
    Stdout(&'a str),
    FromFile(String),
}

impl<'a> SolutionAnswer<'a> {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Stdout(s) => s,
            Self::FromFile(s) => s.trim(),
        }
    }
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

    pub fn parse(s: &'a str) -> anyhow::Result<Test<'a>> {
        let body = match s.strip_prefix("[input]\n") {
            Some(stripped) => stripped,
            None => return Err(anyhow!("`[input]` header is not present")),
        };

        let (input, answer) = match body.split_once("[answer]\n") {
            Some((i, a)) => (i.trim(), a.trim()),
            None => return Err(anyhow!("`[answer]` header is not present")),
        };

        Ok(Self::new(input, answer))
    }

    pub fn run(
        self,
        command: &str,
        output_file: Option<&Path>,
    ) -> anyhow::Result<()> {
        let mut child = create_solution_command(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|| "Couldn't spawn child process")?;

        child
            .stdin
            .take()
            .with_context(|| "Couldn't open child stdin")?
            .write_all(self.input.as_bytes())
            .with_context(|| "Couldn't write to child stdin")?;

        let output = child
            .wait_with_output()
            .with_context(|| "could not read output of the solution")?;
        check_if_solution_terminated_correctly(&output)?;

        let raw_solution_answer = read_soultion_answer(&output, output_file)?;
        let actual_answer = raw_solution_answer.to_str();

        match check_lines(self.answer, actual_answer) {
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
                print!("{}", message);
            }
        }

        Ok(())
    }
}

fn parse_tests(source: &str) -> anyhow::Result<Vec<Test>> {
    source
        .split("[test]\n")
        .filter(|test| !test.is_empty())
        .map(Test::parse)
        .collect()
}

pub fn run(args: Args) -> anyhow::Result<()> {
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
        match test
            .run(args.solution_command.trim(), args.output_file.as_deref())
        {
            Ok(_) => {}
            Err(err) => {
                println!(
                    "{}{}Error occured{}\n{}:",
                    style::Bold,
                    color::Fg(color::Red),
                    style::Reset,
                    err,
                );
                err.chain().skip(1).for_each(|cause| println!("{}", cause));
            }
        }
    }

    Ok(())
}

fn create_solution_command(command: &str) -> std::process::Command {
    let mut solution_splitted = command.split_whitespace();
    let mut solution_command = Command::new(solution_splitted.next().unwrap());
    let solution_args = solution_splitted;
    solution_command.args(solution_args);

    solution_command
}

fn check_if_solution_terminated_correctly(
    output: &Output,
) -> Result<(), anyhow::Error> {
    if !output.status.success() {
        let result = if let Some(libc::SIGSEGV) = output.status.signal() {
            Err(anyhow!("Segmentation fault"))
        } else {
            let solution_stderr = std::str::from_utf8(&output.stderr)
                .with_context(|| {
                    "failed to convert solution stderr to UTF-8"
                })?;
            Err(anyhow!("{}", solution_stderr))
        };

        return result.with_context(|| {
            "Solution terminated with a non-zero exit code".to_string()
        });
    }

    Ok(())
}

fn read_soultion_answer<'a>(
    output: &'a Output,
    output_file: Option<&Path>,
) -> anyhow::Result<SolutionAnswer<'a>> {
    if let Some(file) = output_file {
        let output = std::fs::read_to_string(file).with_context(|| {
            format!("couldn't read solution output from `{}`", file.display())
        })?;
        Ok(SolutionAnswer::FromFile(output))
    } else {
        Ok(SolutionAnswer::Stdout(
            std::str::from_utf8(&output.stdout)
                .with_context(|| "failed to convert solution stdout to UTF-8")?
                .trim(),
        ))
    }
}

fn trim_filter_non_empty(mut line: &str) -> Option<&str> {
    line = line.trim();
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
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

    for i in 1..=max_line_count {
        let cur_line = actual_lines.next().unwrap_or("");
        let cur_correct_line = correct_lines.next().unwrap_or("");

        if i + 1 < 10 {
            message.push_str("  ");
        } else if i + 1 < 100 {
            message.push(' ');
        }
        message.push_str(&format!("{} ", i));

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
        CheckResult::Incorrect { message }
    }
}
