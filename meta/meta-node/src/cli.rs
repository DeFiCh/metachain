use crate::service::EthConfiguration;

/// Available Sealing methods.
#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum Sealing {
	// Seal using rpc method.
	Manual,
	// Seal when transaction is executed.
	Instant,
}

impl Default for Sealing {
	fn default() -> Sealing {
		Sealing::Manual
	}
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[allow(missing_docs)]
	#[command(flatten)]
	pub run: sc_cli::RunCmd,

	/// Choose sealing method.
	#[arg(long, value_enum, ignore_case = true)]
	pub sealing: Option<Sealing>,

	#[command(flatten)]
	pub eth: EthConfiguration,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Key management cli utilities
	#[command(subcommand)]
	Key(sc_cli::KeySubcommand),

	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(sc_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),
	/// Db meta columns information.
	FrontierDb(fc_cli::FrontierDbCmd),
}
