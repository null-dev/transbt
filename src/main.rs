mod cmd;
mod model;
mod util;
mod dump;

fn main() -> eyre::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    stable_eyre::install()?;

    cmd::main()
}
