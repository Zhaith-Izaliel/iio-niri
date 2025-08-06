use anyhow::Result;
use clap::Parser;

mod app;
mod monitor;
mod proxy;

fn main() -> Result<()> {
    let config = app::App::parse();
    proxy::listen_orientation(proxy::INTERFACE, proxy::PATH, config)
}
