use crate::ffi::{DmcTx, DmcBlock};
use crate::service::FullClient;

use tokio::sync::mpsc::Sender;

use std::error::Error;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

lazy_static::lazy_static! {
	pub static ref SIGNAL_HANDLER: RwLock<Option<Sender<()>>> = RwLock::new(None);
	pub static ref NATIVE_HELPER: RwLock<Option<NativeHelper>> = RwLock::new(None);
	pub static ref THREAD_HANDLE: RwLock<Option<JoinHandle<sc_cli::Result<()>>>> = RwLock::new(None);
}

pub struct NativeHelper {
	client: Arc<FullClient>,
}

impl NativeHelper {
	pub fn mint_block<'a, I>(&self, _dmc_txs: I) -> Result<DmcBlock, Box<dyn Error>>
		where I: Iterator<Item = &'a DmcTx>
	{
		let _ = &self.client;
		Ok(DmcBlock {
			payload: vec![1, 2, 3, 4, 5],
		})
	}
}

/// Initialize the native chain helper for FFI usage
pub fn init_helper(client: Arc<FullClient>) {
	log::info!("Initializing FFI helper");
	*NATIVE_HELPER.write().unwrap() = Some(NativeHelper {
		client,
	});
}

/// Setup interrupt handler so that native chain can ask metachain to stop gracefully
pub fn setup_interrupt_handler(sender: Sender<()>) {
	*SIGNAL_HANDLER.write().unwrap() = Some(sender);
}

/// Store thread handle so that we can block later for the thread to exit
pub fn store_handle(handle: JoinHandle<sc_cli::Result<()>>) -> sc_cli::Result<()> {
	*THREAD_HANDLE.write().unwrap() = Some(handle);
	Ok(())  // NOTE: Returns a result so that it's compatible with library and executable
}
