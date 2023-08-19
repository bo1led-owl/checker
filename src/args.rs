use clap::Parser;

#[derive(Parser)]
pub(crate) struct Args {
    #[arg(id = "COMMAND", help = "Command to run the solution")]
    pub solution_command: String,

    #[arg(id = "ARGS", help = "Arguments passed to the solution")]
    pub solution_arguments: Vec<String>,

    #[arg(
        value_name = "COMMAND",
        short,
        long,
        help = "Command to build the solution before tests"
    )]
    pub build_rule: Option<String>,

    #[arg(short, long, help = "Print full output of the solution")]
    pub full_output: bool,

    #[arg(
        value_name = "FILE",
        short,
        long,
        default_value = None,
        help = "Path to the file with test input"
    )]
    pub input: Option<String>,

    #[arg(
        value_name = "FILE",
        short,
        long,
        help = "Path to the file with correct output"
    )]
    pub answer: Option<String>,
}
