use std::path::PathBuf;

use anyhow::{anyhow, Context};
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

fn add_test(mut test_suite: String, input: &str, answer: &str) -> String {
    test_suite.push_str(&format!("{}", checker::Test { input, answer }));
    return test_suite;
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut tests = std::fs::read_to_string(&args.test_suite).with_context(|| {
        format!(
            "Couldn't read test suite description from `{}`",
            args.test_suite.display()
        )
    })?;

    match args.command {
        Command::Run { solution_command } => run(tests, solution_command),
        Command::AddTest { input, answer } => {
            tests = add_test(tests, &input, &answer);
            std::fs::write(&args.test_suite, tests).with_context(|| {
                format!("Failed to write tests into `{}`", args.test_suite.display())
            })
        }
        Command::ClearTests => std::fs::write(&args.test_suite, "")
            .with_context(|| format!("Failed to write into `{}`", args.test_suite.display())),
    }
}
