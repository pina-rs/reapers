//! What goes wrong reading a record.

use thiserror::Error;

/// A record that could not be read: a genome, an NFT id, a weapon id, a hunt
/// log, a save. Nothing malformed is ever taken for something real.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("{message}{}", .input.as_ref().map(|input| format!(": {input:?}")).unwrap_or_default())]
pub struct FormatError {
	/// What was wrong.
	pub message: String,

	/// The text that was being read, when there was one.
	pub input: Option<String>,
}

impl FormatError {
	pub fn new(message: impl Into<String>) -> Self {
		Self {
			message: message.into(),
			input: None,
		}
	}

	pub fn with_input(message: impl Into<String>, input: impl Into<String>) -> Self {
		Self {
			message: message.into(),
			input: Some(input.into()),
		}
	}
}
