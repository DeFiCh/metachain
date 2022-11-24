//! RPC Node Template CLI library.
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

use clap::Parser;

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;

mod native {
	pub fn init_helper(_client: std::sync::Arc<crate::service::FullClient>) {}
	pub fn setup_interrupt_handler(_sender: tokio::sync::mpsc::Sender<()>) {}
	pub fn store_handle(handle: std::thread::JoinHandle<sc_cli::Result<()>>) -> sc_cli::Result<()> {
		handle.join().unwrap()
	}
}

fn main() -> sc_cli::Result<()> {
	let cli = cli::Cli::parse();
	command::run(cli)
}
