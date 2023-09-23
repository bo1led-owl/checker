use std::{
    fs::File,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use termion::color;

#[derive(Parser)]
pub struct Checker {
    #[arg(
        value_name = "COMMAND",
        short,
        long,
        default_value = None,
        value_parser = clap::builder::NonEmptyStringValueParser::new(),
        help = "Command to build the solution before tests",
    )]
    build_rule: Option<String>,

    #[arg(short, long, help = "Print full output of the solution")]
    full_output: bool,

    #[arg(
        value_name = "FILE",
        short,
        long,
        default_value = None,
        help = "Path to the file with test input"
    )]
    input_path: Option<PathBuf>,

    #[arg(id = "ANSWER", help = "Path to the file with correct output")]
    answer_path: PathBuf,

    #[arg(id = "SOLUTION", help = "Command to run the solution")]
    solution: String,
}

impl Default for Checker {
    fn default() -> Self {
        Self::parse()
    }
}

impl Checker {
    pub fn run(&self) -> Result<()> {
        let input = self.create_input_pipe()?;

        let correct_answer = self.get_correct_answer()?;
        let trimmed_correct_answer = correct_answer.trim();

        self.run_build_rule()?;

        let output = self.run_solution(input)?;
        let trimmed_actual_answer = output.trim();

        if self.full_output {
            println!("Output:\n{}", trimmed_actual_answer);
        }

        let correct = Self::check_line_count(
            trimmed_correct_answer,
            trimmed_actual_answer,
        ) && Self::check_lines(
            trimmed_correct_answer,
            trimmed_actual_answer,
        );

        if correct {
            println!("{}Tests passed", color::Fg(color::Green));
        }
        Ok(())
    }

    fn create_input_pipe(&self) -> Result<Stdio> {
        match &self.input_path {
            Some(path) => {
                File::open(path).map(Stdio::from).with_context(|| {
                    format!(
                        "could not read test input from `{}`",
                        path.display()
                    )
                })
            }
            None => Ok(Stdio::null()),
        }
    }

    fn get_correct_answer(&self) -> Result<String> {
        std::fs::read_to_string(&self.answer_path).with_context(|| {
            format!(
                "could not read correct answer from `{}`",
                self.answer_path.display()
            )
        })
    }

    /// Runs the solution and returns its output.
    fn run_solution(&self, input: Stdio) -> Result<String> {
        let mut solution = self.solution.split_whitespace();
        let mut command = Command::new(solution.next().unwrap());
        let output =
            command.args(solution).stdin(input).output().with_context(
                || {
                    format!(
                        "could not read output of the solution `{}`",
                        self.solution
                    )
                },
            )?;

        if !output.status.success() {
            Err(
                anyhow!(String::from_utf8_lossy(&output.stderr).into_owned())
                    .context(format!(
                        "could not read output of the solution `{}`",
                        self.solution
                    )),
            )
        } else {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        }
    }

    /// Runs build rule and checks if it exits successfully
    fn run_build_rule(&self) -> Result<()> {
        if self.build_rule.is_none() {
            return Ok(());
        }

        if let Some(build_rule) = self.build_rule.as_ref() {
            if build_rule.is_empty() {
                return Err(anyhow!("Build rule is empty")
                    .context("could not execute build rule"));
            }

            let mut build_rule_splitted = build_rule.split_whitespace();
            let output = Command::new(build_rule_splitted.next().unwrap())
                .args(build_rule_splitted)
                .output()
                .with_context(|| {
                    format!("could not execute build rule `{}`", build_rule)
                })?;

            if !output.status.success() {
                return Err(anyhow!(
                    String::from_utf8_lossy(&output.stderr).into_owned()
                )
                .context(format!(
                    "could not execute build rule `{}`",
                    build_rule,
                )));
            }
        }

        Ok(())
    }

    fn check_line_count(correct_answer: &str, actual_answer: &str) -> bool {
        let correct_line_count = correct_answer.lines().count();
        let actual_line_count = actual_answer.lines().count();

        if correct_line_count != actual_line_count {
            println!(
                "{}Number of lines differs:{} expected {}, got {}",
                color::Fg(color::Red),
                color::Fg(color::Reset),
                correct_line_count,
                actual_line_count
            );
            false
        } else {
            true
        }
    }

    fn check_lines(correct_answer: &str, actual_answer: &str) -> bool {
        let mut res = false;
        let mut correct_lines = correct_answer.lines();

        actual_answer.lines().enumerate().for_each(|(i, cur_line)| {
            let cur_correct_line = correct_lines.next().unwrap();

            if cur_line != cur_correct_line {
                println!(
                    "{}Line {} differs:{} expected {}, got {}",
                    color::Fg(color::Red),
                    i + 1,
                    color::Fg(color::Reset),
                    cur_correct_line,
                    cur_line
                );
                res = false;
            }
        });

        res
    }
}
