use std::process::{Command, Stdio};

use clap::Parser;
use termion::color;

#[derive(Parser)]
struct Args {
    #[arg(id = "BINARY", help = "Path to the executable")]
    pub binary: String,

    #[arg(
        value_name = "FILE", 
        short, 
        long, 
        default_value = None, 
        help = "Path to the file with test input. If not specified, no input is provided"
    )]
    pub input: Option<String>,

    #[arg(
        value_name = "FILE",
        short,
        long,
        default_value = "answer.txt",
        help = "Path to the file with correct output."
    )]
    pub answer: String,
}

pub struct Checker {
    args: Args,
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}

impl Checker {
    pub fn new() -> Self {
        Self {
            args: Args::parse(),
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let input = self.get_input();

        if let Err(err) = input {
            return Err(format!(
                "Error reading test input from {}:\n{}",
                self.args.answer, err
            ));
        }

        let input = input.unwrap();

        let correct_answer = std::fs::read_to_string(&self.args.answer);
        if let Err(err) = correct_answer {
            return Err(format!(
                "Error reading correct answer from {}:\n{}",
                self.args.answer, err
            ));
        }

        let correct_answer = correct_answer.unwrap();
        let trimmed_correct_answer = correct_answer.trim();

        let output = Command::new(&self.args.binary).stdin(input).output();

        if let Err(err) = output {
            return Err(format!(
                "Error reading output of the binary {}:\n{}",
                self.args.binary, err
            ));
        }

        let output = output.unwrap();
        let actual_answer = String::from_utf8_lossy(&output.stdout);
        let trimmed_actual_answer = actual_answer.trim();

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

    fn get_input(&self) -> std::io::Result<Stdio> {
        match &self.args.input {
            Some(path) => std::fs::File::open(path).map(Stdio::from),
            None => Ok(Stdio::null()),
        }
    }

    fn check_line_count(correct_answer: &str, actual_answer: &str) -> bool {
        let correct_line_count = correct_answer.lines().count();
        let actual_line_count = actual_answer.lines().count();

        if correct_line_count != actual_line_count {
            eprintln!(
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

        let mut correct_line = correct_answer.lines();
        for (i, line) in actual_answer.lines().enumerate() {
            let cur_correct_line = correct_line.next().unwrap();
            if line != cur_correct_line {
                res = false;
                eprintln!(
                    "{}Line {} differs:\n{}expected: {}\ngot: {}\n",
                    color::Fg(color::Red),
                    i + 1,
                    color::Fg(color::Reset),
                    cur_correct_line,
                    line
                );
            }
        }

        res
    }
}
