mod cli;
mod errors;
mod server;

use clap::Parser;
use cli::Arguments;
use server::build_server;
use std::net::SocketAddr;
use tracing::{subscriber::set_global_default, Level};

#[tokio::main]
async fn main() {
    let arguments = Arguments::parse();

    set_verbosity(arguments.verbose);

    let redis_path = format!("redis://{}:{}/", arguments.redis_host, arguments.redis_port);
    let client = redis::Client::open(redis_path).expect("Unable to connect to Redis");
    let router = build_server(client);
    let address = SocketAddr::from((arguments.server_host, arguments.server_port));

    tracing::debug!("Server listening on {}.", address);

    axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
        .expect("Unable to start server.");
}

fn set_verbosity(occurrences: usize) {
    let level = match occurrences {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let collector = tracing_subscriber::fmt().with_max_level(level).finish();

    set_global_default(collector).expect("Unable to set global default tracing collector.")
}
