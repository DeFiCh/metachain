//! RPC Node Template CLI library.
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

use meta_core;

fn main() -> sc_cli::Result<()> {
	meta_core::run()
}
