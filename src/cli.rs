use clap::{AppSettings, Parser};
use std::fmt;
use std::io;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;

#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Arguments {
    #[clap(short, long, parse(from_occurrences))]
    /// Make the subcommand more talkative.
    pub verbose: usize,
    #[clap(short('p'), long, default_value = "3000")]
    /// The port for the Key-Value server to listen on.
    pub server_port: u16,
    #[clap(long, default_value = "6379")]
    /// The port for the Key-Value server to connect to Redis on.
    pub redis_port: u16,
    #[clap(short('s'), long, default_value = "localhost")]
    /// The address for the Key-Value server to listen on.
    pub server_host: Host,
    #[clap(short('r'), long, default_value = "localhost")]
    /// The address for the Key-Value server to connect to Redis on.
    pub redis_host: Host,
    #[clap(short, long, default_value = "1024")]
    /// Limit the max number of in-flight requests.
    /// A request is in-flight from the time the request is received until the response future completes.
    /// This includes the time spent in the next layers.
    pub concurrency_limit: usize,
    #[clap(short, long, default_value = "10000")]
    /// Fail requests that take longer than timeout.
    pub timeout_in_millis: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Host {
    address: IpAddr
}

impl FromStr for Host {
    type Err = io::Error; 
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut addresses = format!("{}:0", s).as_str().to_socket_addrs()?;
        let address: io::Result<SocketAddr> = addresses.next().ok_or(io::ErrorKind::AddrNotAvailable.into());

        Ok(Host {
            address: address?.ip()
        })
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

impl From<Host> for IpAddr {
    fn from(host: Host) -> Self {
        host.address
    }
}