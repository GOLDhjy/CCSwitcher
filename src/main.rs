fn main() {
    if let Err(err) = ccswitcher::run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
