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

impl Checker {
    pub fn run(&self) -> Result<()> {
        let input = self.create_input_pipe()?;

        let correct_answer = self.get_correct_answer()?;
        let trimmed_correct_answer = correct_answer.trim();

        let solution_output = self.run_solution(input)?;
        let trimmed_actual_answer = solution_output.trim();

        if self.print_output {
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
            println!(
                "{}Tests passed{}",
                color::Fg(color::Green),
                color::Fg(color::Reset)
            );
        }

        Ok(())
    }

    fn create_input_pipe(&self) -> Result<Stdio> {
        self.input_file.as_ref().map_or(Ok(Stdio::null()), |path| {
            File::open(path).map(Stdio::from).with_context(|| {
                format!("could not read test input from `{}`", path.display())
            })
        })
    }

    fn get_correct_answer(&self) -> Result<String> {
        std::fs::read_to_string(&self.answer_file).with_context(|| {
            format!(
                "could not read correct answer from `{}`",
                self.answer_file.display()
            )
        })
    }

    /// Runs the solution and returns its output.
    fn run_solution(&self, input: Stdio) -> Result<String> {
        let mut solution_splitted = self.solution_command.split_whitespace();
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
                    self.solution_command
                )
            })?;

        let solution_stdout =
            String::from_utf8_lossy(&solution_output.stdout).into_owned();
        if !solution_output.status.success() {
            let solution_stderr =
                String::from_utf8_lossy(&solution_output.stderr);

            Err(anyhow!(
                "Stdout:\n{}\nStderr:\n{}",
                solution_stdout,
                solution_stderr
            ))
            .with_context(|| {
                format!(
                    "solution `{}` terminated with a non-zero exit code",
                    self.solution_command
                )
            })
        } else {
            let output = if let Some(path) = self.output_file.as_ref() {
                std::fs::read_to_string(path).with_context(|| {
                    format!(
                        "could not read output of the solution from `{}`",
                        path.display()
                    )
                })?
            } else {
                solution_stdout
            };
            Ok(output)
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

            if cur_line.trim() != cur_correct_line.trim() {
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
