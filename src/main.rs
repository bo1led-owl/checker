use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = checker::Args::parse();
    checker::run(args)
}
