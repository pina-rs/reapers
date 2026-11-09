//! The repository's own tools. Run them from anywhere inside the repository:
//!
//! ```sh
//! cargo xtask check            # format, lints, every build and every test
//! cargo xtask check --fast     # the same without the PostgreSQL-backed server tests
//! cargo xtask hooks install    # once per clone: guard main
//! cargo xtask merge            # push, open the pull request, check, land it on main
//! cargo xtask leptoskit        # move every leptoskit pin to leptoskit main's head
//! cargo xtask build-art        # rebuild every model and portrait (Blender, gltfpack)
//! cargo xtask build-music      # fetch and re-encode the music (curl, ffmpeg)
//! cargo xtask fetch-depth-model # fetch the phones' depth model and its licence (curl)
//! ```
//!
//! `cargo xtask` is an alias in `.cargo/config.toml`.

mod art;
mod check;
mod depth;
mod git;
mod hooks;
mod leptoskit;
mod merge;
mod music;
mod run;

use std::process::ExitCode;

use clap::Parser;
use clap::Subcommand;

use crate::run::Failure;

#[derive(Parser)]
#[command(name = "cargo xtask", about = "The Reapers repository's own tools.")]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Run every check a change has to pass before it reaches main.
	Check {
		/// Skip the server's tests, which start a PostgreSQL cluster.
		#[arg(long)]
		fast: bool,
	},
	/// Guard main against changes the checks have not passed.
	Hooks {
		#[command(subcommand)]
		action: Option<hooks::Action>,
	},
	/// Land this branch on main through a pull request.
	Merge,
	/// Move every leptoskit pin to one revision.
	Leptoskit(leptoskit::Options),
	/// Build every 3D model and NFT portrait from `art/blender`.
	BuildArt(art::Options),
	/// Fetch the music from its makers and re-encode it.
	BuildMusic,
	/// Fetch the depth model phones without platform depth read rooms with.
	FetchDepthModel,
}

fn main() -> ExitCode {
	let cli = Cli::parse();
	let root = run::root();
	let outcome = match cli.command {
		Command::Check { fast } => check::main(&root, fast),
		Command::Hooks { action } => hooks::main(&git::Git::at(&root), action.unwrap_or_default()),
		Command::Merge => merge::main(&root),
		Command::Leptoskit(options) => leptoskit::main(&root, &options),
		Command::BuildArt(options) => art::main(&root, &options),
		Command::BuildMusic => music::main(&root),
		Command::FetchDepthModel => depth::main(&root),
	};

	match outcome {
		Ok(()) => ExitCode::SUCCESS,
		Err(Failure { message, code }) => {
			if !message.is_empty() {
				eprintln!("{message}");
			}
			ExitCode::from(code)
		}
	}
}
