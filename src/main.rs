mod err;
mod opt;
mod server;

use structopt::StructOpt;

fn main() -> Result<(), err::DisplayError> {
    let opt::Options { verbose, port } = opt::Options::from_args();

    env_logger::Builder::new()
        .filter_level(match verbose {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .init();

    server::run(&([0, 0, 0, 0], port).into())?;

    Ok(())
}
