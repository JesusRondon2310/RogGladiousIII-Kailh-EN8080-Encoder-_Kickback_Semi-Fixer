mod filter;
mod helpers;

fn main() {
    if filter::run().is_err() {
        std::process::exit(1);
    }
}
