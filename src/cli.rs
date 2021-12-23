use clap::{AppSettings, Parser};
use std::net::IpAddr;

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
pub struct Arguments {
    #[clap(short, long, parse(from_occurrences))]
    /// Make the subcommand more talkative.
    pub verbose: usize,
    #[clap(short('p'), long, default_value = "3000")]
    /// The port for the Key-Value server to liston on.
    pub server_port: u16,
    #[clap(long, default_value = "6379")]
    /// The port for the Key-Value server to connect to Redis on.
    pub redis_port: u16,
    #[clap(short('s'), long, default_value = "127.0.0.1")]
    /// The address for the Key-Value server to liston on.
    pub server_host: IpAddr,
    #[clap(short('r'), long, default_value = "127.0.0.1")]
    /// The address for the Key-Value server to connect to Redis on.
    pub redis_host: IpAddr,
}
