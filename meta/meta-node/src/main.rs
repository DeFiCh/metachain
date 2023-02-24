//! RPC Node Template CLI library.
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod account_key;
mod chain_spec;
mod cli;
mod client;
mod command;
mod eth;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
	command::run()
}
