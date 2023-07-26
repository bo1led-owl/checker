use checker::Checker;

fn main() {
    if let Err(err) = Checker::new().run() {
        eprintln!("{err}");
    }
}
