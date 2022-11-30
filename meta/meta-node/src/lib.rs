mod chain_spec;
mod native;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;

use std::error::Error;

use clap::{CommandFactory, ErrorKind, FromArgMatches};
use cli::Cli;
use cxx::{CxxString, CxxVector};

// NOTE: Struct definitions for FFI should match those defined in libain-rs/protobuf

#[cxx::bridge]
mod ffi {
    pub struct DmcTx {
        pub from: String,
        pub to: String,
        pub amount: i64,
    }

    pub struct DmcBlock {
        pub payload: Vec<u8>,
    }

    struct ExecResult {
        is_help: bool,
        success: bool,
        daemon: bool,
    }

    extern "Rust" {
        fn connect_block(payload: ffi::DmcBlock) -> Result<()>;
        fn mint_block(dmc_txs: &CxxVector<DmcTx>) -> Result<DmcBlock>;
        fn parse_args_and_run(args: &CxxVector<CxxString>) -> ExecResult;
        fn interrupt_dmc() -> Result<()>;
    }
}
#[inline]
fn connect_block(payload: ffi::DmcBlock) -> sc_cli::Result<()>{
    return Ok(());
}

#[inline]
fn mint_block(dmc_txs: &CxxVector<ffi::DmcTx>) -> Result<ffi::DmcBlock, Box<dyn Error>> {
	self::native::NATIVE_HELPER
         .read().unwrap().as_ref().expect("uninitialized native helper")
         .mint_block(dmc_txs.iter())
}

#[inline]
fn interrupt_dmc() -> sc_cli::Result<()> {
    log::info!("Signaling metachain node to terminate");
    let _ = self::native::SIGNAL_HANDLER.read().unwrap().as_ref().expect("uninitialized signal handler")
        .blocking_send(());
    log::info!("Waiting for processes to finish");
    self::native::THREAD_HANDLE.write().unwrap().take().expect("thread not spawned yet?")
        .join().unwrap()
}

fn parse_args_and_run(args: &CxxVector<CxxString>) -> ffi::ExecResult {
    let args: Vec<String> = args
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let res = Cli::command().try_get_matches_from(args)
        .and_then(|mut m| Cli::from_arg_matches_mut(&mut m));

    let cli = match res {
        Ok(c) => c,
        Err(e) => {
            let _ = e.print();
            let is_help = match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => true,
                _ => false,
            };
            return ffi::ExecResult {
                is_help,
                success: is_help,
                daemon: false,
            }
        },
    };

    let mut result = ffi::ExecResult {
        is_help: false,
        success: false,
        daemon: cli.subcommand.is_none(),
    };
    match command::run(cli) {
        Ok(_) => {
            // Deciding to run the node, give control back to defid without joining handle
            // FIXME: Figure out how to send SIGINT on defid exiting for cleanly shutting down
            result.success = true;
        },
        Err(e) => eprintln!("{:?}", e),
    }

    result
}
