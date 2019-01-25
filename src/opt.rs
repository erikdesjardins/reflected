use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Options {
    #[structopt(
        short = "v",
        long = "verbose",
        parse(from_occurrences),
        raw(global = "true"),
        help = "Logging verbosity (-v info, -vv debug, -vvv trace)"
    )]
    pub verbose: u8,

    #[structopt(help = "Port to listen on")]
    pub port: u16,
}