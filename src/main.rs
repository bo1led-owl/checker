use std::{io::Write, path::PathBuf};

use anyhow::{anyhow, Context};
use checker::Test;
use clap::{Parser, Subcommand};
use termion::{color, style};

#[derive(Parser)]
pub struct Args {
    #[arg(
        long,
        help = "Path to the file with test suite description",
        default_value = "tests"
    )]
    test_suite: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run {
        #[arg(id = "SOLUTION", help = "Command to run the solution")]
        solution_command: String,
    },
    AddTest {
        #[arg(id = "INPUT", help = "Data passed as input")]
        input: String,
        #[arg(id = "ANSWER", help = "Expected output")]
        answer: String,
    },
    ClearTests,
}

fn run(test_suite: String, solution_command: String) -> anyhow::Result<()> {
    if solution_command.is_empty() {
        return Err(anyhow!("Empty solution command provided"));
    }

    let tests = checker::parse_tests(&test_suite)?;

    for (i, test) in tests.into_iter().enumerate() {
        print!("{}Test {}: {}", style::Bold, i + 1, style::Reset);
        match checker::run_test(test, solution_command.trim()) {
            Ok(_) => {}
            Err(err) => {
                println!(
                    "{}{}Error occured{}",
                    style::Bold,
                    color::Fg(color::Red),
                    style::Reset,
                );
                println!("{err}:");
                err.chain().skip(1).for_each(|cause| eprintln!("{}", cause));
            }
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Run { solution_command } => {
            let tests = std::fs::read_to_string(&args.test_suite).with_context(|| {
                format!(
                    "Couldn't read test suite description from `{}`",
                    args.test_suite.display()
                )
            })?;

            run(tests, solution_command)
        }
        Command::AddTest { input, answer } => {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(&args.test_suite)
                .with_context(|| format!("Error opening `{}`", args.test_suite.display()))?;

            file.write_all(
                format!(
                    "{}",
                    Test {
                        input: &input,
                        answer: &answer
                    }
                )
                .as_bytes(),
            )
            .with_context(|| format!("Failed to write tests into `{}`", args.test_suite.display()))
        }
        Command::ClearTests => std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&args.test_suite)
            .map(|_| ())
            .with_context(|| format!("Error opening `{}`", args.test_suite.display())),
    }
}
