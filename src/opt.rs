use std::net::SocketAddr;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about)]
pub struct Options {
    #[structopt(
        short = "v",
        long = "verbose",
        parse(from_occurrences),
        global = true,
        help = "Logging verbosity (-v info, -vv debug, -vvv trace)"
    )]
    pub verbose: u8,

    #[structopt(help = "Socket address to listen on")]
    pub listen_addr: SocketAddr,
}
