use checker::Checker;

fn main() {
    if let Err(error) = Checker::default().run() {
        eprintln!("{error}");
    }
}
