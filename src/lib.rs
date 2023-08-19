mod args;
pub mod error;

use args::Args;
use error::CheckerError;

use std::{
    fs::File,
    process::{Command, Stdio},
};

use clap::Parser;
use termion::color;

pub struct Checker {
    args: Args,
}

impl Default for Checker {
    fn default() -> Self {
        Self {
            args: Args::parse(),
        }
    }
}

impl Checker {
    pub fn run(&self) -> Result<(), CheckerError> {
        let input = self.get_input()?;

        let correct_answer = self.get_correct_answer()?;
        let trimmed_correct_answer = correct_answer.trim();

        self.run_build_rule()?;

        let output = self.run_solution(input)?;
        let trimmed_actual_answer = output.trim();

        if self.args.full_output {
            println!("Output:\n{}", trimmed_actual_answer);
        }

        if !Self::check_line_count(
            trimmed_correct_answer,
            trimmed_actual_answer,
        ) || !Self::check_lines(
            trimmed_correct_answer,
            trimmed_actual_answer,
        ) {
            return Ok(());
        }

        println!("{}Tests passed", color::Fg(color::Green));
        Ok(())
    }

    fn get_input(&self) -> Result<Stdio, CheckerError> {
        let open_stdio = |path| {
            File::open(path)
                .map(Stdio::from)
                .map_err(|err| CheckerError {
                    error: format!("Error reading test input from {path}:"),
                    error_description: Some(err.to_string()),
                })
        };

        match &self.args.input {
            Some(path) => open_stdio(path),
            None => Ok(Stdio::null()),
        }
    }

    fn get_correct_answer(&self) -> Result<String, CheckerError> {
        let get_answer = |path| {
            std::fs::read_to_string(path).map_err(|err| CheckerError {
                error: format!("Error reading correct answer from {path}:"),
                error_description: Some(err.to_string()),
            })
        };

        match &self.args.answer {
            Some(path) => get_answer(path),
            None => Ok(String::new()),
        }
    }

    fn run_solution(&self, input: Stdio) -> Result<String, CheckerError> {
        match Command::new(&self.args.solution_command)
            .args(&self.args.solution_arguments)
            .stdin(input)
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    Err(CheckerError {
                        error: "Solution didn't exit successfully:".to_string(),
                        error_description: Some(format!(
                            "{}\n{}",
                            &output.status,
                            String::from_utf8_lossy(&output.stderr)
                        )),
                    })
                } else {
                    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
                }
            }
            Err(err) => Err(CheckerError {
                error: format!(
                    "Error reading output of the solution {}:",
                    self.args.solution_command,
                ),
                error_description: Some(err.to_string()),
            }),
        }
    }

    fn run_build_rule(&self) -> Result<(), CheckerError> {
        if let Some(build_rule) = &self.args.build_rule {
            if build_rule.is_empty() {
                return Err(CheckerError {
                    error: "Cannot use empty build rule".to_string(),
                    error_description: None,
                });
            }

            let mut build_rule = build_rule.split_whitespace();
            match Command::new(build_rule.next().unwrap())
                .args(build_rule)
                .output()
            {
                Ok(output) => {
                    if !output.status.success() {
                        return Err(CheckerError {
                            error: "Build didn't complete successfully:"
                                .to_string(),
                            error_description: Some(format!(
                                "{}\n{}",
                                &output.status,
                                String::from_utf8_lossy(&output.stderr),
                            )),
                        });
                    }
                }
                Err(err) => {
                    return Err(CheckerError {
                        error: "Error running build:".to_string(),
                        error_description: Some(err.to_string()),
                    })
                }
            }
        }

        Ok(())
    }

    fn check_line_count(correct_answer: &str, actual_answer: &str) -> bool {
        let correct_line_count = correct_answer.lines().count();
        let actual_line_count = actual_answer.lines().count();

        if correct_line_count != actual_line_count {
            println!(
                "{}Number of lines differs:\n{}expected {}, got {}",
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
        let mut res = true;
        let mut correct_lines = correct_answer.lines();

        actual_answer.lines().enumerate().for_each(|(i, line)| {
            // unwrap is safe because the test inly runs if the line count is equal
            let cur_correct_line =
                unsafe { correct_lines.next().unwrap_unchecked() };

            if line != cur_correct_line {
                res = false;
                println!(
                    "{}Line {} differs:\n{}expected: {}\ngot: {}",
                    color::Fg(color::Red),
                    i + 1,
                    color::Fg(color::Reset),
                    cur_correct_line,
                    line
                );
            }
        });

        res
    }
}
