fn main() {
    if let Err(err) = kernel::fs::init() {
        eprintln!("kernel init failed: {} ({})", err.name(), err.as_isize());
    }
}
