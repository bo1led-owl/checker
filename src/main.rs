use checker::Checker;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    Checker::parse().run()
}
