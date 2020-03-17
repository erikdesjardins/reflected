mod as_ref;
mod body;
mod err;
mod file;
mod opt;
mod routes;
mod server;

use structopt::StructOpt;

fn main() -> Result<(), err::DisplayError> {
    let opt::Options {
        verbose,
        listen_addr,
    } = opt::Options::from_args();

    env_logger::Builder::new()
        .filter_level(match verbose {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .init();

    server::run(&listen_addr)?;

    Ok(())
}
