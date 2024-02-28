use clap::Parser;

const MILLISECOND: f32 = 1.0 / 1000.0;

fn format_time(time: f32) -> String {
    if time < 10.0 * MILLISECOND {
        "less than 10ms".to_string()
    } else {
        format!("{:.2}s", time)
    }
}

fn main() -> anyhow::Result<()> {
    let start = std::time::SystemTime::now();
    let args = checker::Args::parse();
    checker::run(args)?;
    println!(
        "{}Finished in {}{}",
        termion::style::Bold,
        format_time(start.elapsed()?.as_secs_f32()),
        termion::style::Reset,
    );
    Ok(())
}
