//! Tests that need the compiled program artifact.
//!
//! The program crate is `cdylib` only so the deployed binary can use fat LTO.
//! Cargo cannot link a `cdylib` into an integration test, which means this
//! file cannot `use` the program's types. Two patterns cover that gap:
//!
//! 1. Behavioural coverage lives in `src/lib.rs` under `#[cfg(test)] mod
//!    tests`, next to the code it exercises. Run it with `cargo test --lib`;
//!    it measures coverage of the real program source.
//! 2. On-chain behaviour lives in `tests/surfpool`, which `pina test` runs
//!    against the built SBF artifact.
//!
//! Keep this file for anything that only needs the compiled artifact path.

/// Pina's build publishes the deploy artifact for `pina test`.
#[test]
fn deploy_artifact_path_is_conventional() {
	let artifact = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("target/deploy")
		.join(env!("CARGO_PKG_NAME"));

	assert!(
		artifact.extension().is_none(),
		"artifact base path is {}",
		artifact.display()
	);
}
