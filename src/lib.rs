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

    #[arg(id = "answer", help = "Path to the file with correct output")]
    answer_path: PathBuf,

    #[arg(id = "solution", help = "Command to run the solution")]
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
        self.input_path.as_ref().map_or(Ok(Stdio::null()), |path| {
            File::open(path).map(Stdio::from).with_context(|| {
                format!("could not read test input from `{}`", path.display())
            })
        })
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
        let mut solution_splitted = self.solution.split_whitespace();
        let mut solution_command =
            Command::new(solution_splitted.next().unwrap());
        let solution_args = solution_splitted;

        let solution_output = solution_command
            .args(solution_args)
            .stdin(input)
            .output()
            .with_context(|| {
                format!(
                    "could not read output of the solution `{}`",
                    self.solution
                )
            })?;

        if !solution_output.status.success() {
            let solution_stderr =
                String::from_utf8_lossy(&solution_output.stderr).into_owned();

            Err(anyhow!(solution_stderr)).with_context(|| {
                format!(
                    "solution `{}` terminated with a non-zero exit code",
                    self.solution
                )
            })
        } else {
            let output =
                String::from_utf8_lossy(&solution_output.stdout).into_owned();
            Ok(output)
        }
    }

    /// Runs build rule and checks if it terminates successfully
    fn run_build_rule(&self) -> Result<()> {
        if self.build_rule.is_none() {
            return Ok(());
        }

        let build_rule = self.build_rule.as_ref().unwrap();

        let mut build_rule_splitted = build_rule.split_whitespace();
        let build_rule_command = build_rule_splitted.next().unwrap();
        let build_rule_args = build_rule_splitted;

        let build_rule_output = Command::new(build_rule_command)
            .args(build_rule_args)
            .output()
            .with_context(|| {
                format!("could not execute build rule `{build_rule}`")
            })?;

        if !build_rule_output.status.success() {
            let build_rule_stderr =
                String::from_utf8_lossy(&build_rule_output.stderr).into_owned();

            Err(anyhow!(build_rule_stderr)).with_context(|| {
                format!(
                    "build rule `{build_rule}` terminated with a non-zero exit code",
                )
            })
        } else {
            Ok(())
        }
    }

    fn check_line_count(correct_answer: &str, actual_answer: &str) -> bool {
        let correct_line_count = correct_answer.lines().count();
        let actual_line_count = actual_answer.lines().count();

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
        let mut correct_lines = correct_answer.lines();

        for (i, cur_line) in actual_answer.lines().enumerate() {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_line_count() {
        let str1 = "1\n2";
        let str2 = "1\n5";
        let str3 = "1\n2\n3";

        assert!(Checker::check_line_count(str1, str2));
        assert!(!Checker::check_line_count(str1, str3));
    }

    #[test]
    fn check_lines() {
        let str1 = "1\n2";
        let str2 = "1\n55";

        assert!(Checker::check_lines(str1, str1));
        assert!(!Checker::check_lines(str1, str2));
    }
}
