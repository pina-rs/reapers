//! Writing and checking migrations, as `leptoskit db migration create` and
//! `leptoskit db check` do, for a keeper built without the leptoskit CLI.
//!
//! Each migration is a directory under `migrations/`, listed in order in
//! `migrations/registry.txt`, holding the SQL that moves the database
//! forward (`migration.sql`), the SQL that moves it back (`down.sql`), and
//! the schema it leaves (`schema.json`). The next migration is the
//! difference between the latest `schema.json` and the tables in the code.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use leptoskit_db_schema::Dialect;
use leptoskit_db_schema::NoRenames;
use leptoskit_db_schema::RequireHints;
use leptoskit_db_schema::Schema;

/// Why a migration could not be written or checked.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct MigrationError(String);

fn fail(message: impl Into<String>) -> MigrationError {
	MigrationError(message.into())
}

/// The schema the keeper's tables add up to, on Postgres.
///
/// # Errors
///
/// When the tables don't make a valid schema.
pub fn target() -> Result<Schema, MigrationError> {
	leptoskit_db::schema(Dialect::Postgres)
		.map(|schema| schema.without_rename_hints())
		.map_err(|error| fail(format!("the tables don't make a schema: {error}")))
}

/// The migrations `directory` lists, in order.
///
/// # Errors
///
/// When the registry cannot be read.
pub fn names(directory: &Path) -> Result<Vec<String>, MigrationError> {
	let registry = directory.join("registry.txt");
	let text = fs::read_to_string(&registry).unwrap_or_default();

	Ok(text
		.lines()
		.map(str::trim)
		.filter(|line| !line.is_empty() && !line.starts_with('#'))
		.map(str::to_owned)
		.collect())
}

/// The schema the latest migration in `directory` leaves, or an empty one
/// before the first.
///
/// # Errors
///
/// When the latest migration's `schema.json` is missing or unreadable.
pub fn latest(directory: &Path) -> Result<Schema, MigrationError> {
	let Some(name) = names(directory)?.pop() else {
		return Ok(Schema::empty(Dialect::Postgres));
	};
	let path = directory.join(&name).join("schema.json");
	let json =
		fs::read_to_string(&path).map_err(|error| fail(format!("{}: {error}", path.display())))?;

	Schema::from_json(&json).map_err(|error| fail(format!("{}: {error}", path.display())))
}

/// Checks the latest migration in `directory` leaves exactly the tables in
/// the code: nothing changed without a migration.
///
/// It compares the latest `schema.json` with the code's schema as text, so
/// it holds in a build where `serde_json` reads numbers with arbitrary
/// precision, as the keeper's does; only naming the changes needs the
/// snapshot read back.
///
/// # Errors
///
/// When a table changed since the latest migration, naming the changes when
/// it can.
pub fn check(directory: &Path) -> Result<(), MigrationError> {
	let target = target()?;
	let written = match names(directory)?.pop() {
		Some(name) => {
			let path = directory.join(&name).join("schema.json");

			fs::read_to_string(&path)
				.map_err(|error| fail(format!("{}: {error}", path.display())))?
		}
		None => String::new(),
	};

	if written.trim() == target.to_json().trim() {
		return Ok(());
	}

	let mut message = String::from(
		"the tables changed since the latest migration; run `cargo run -p reapers_db --bin \
		 reapers-migration -- <name>`",
	);

	if let Ok(plan) = latest(directory).and_then(|latest| {
		leptoskit_db_schema::diff(&latest, &target, &mut NoRenames)
			.map_err(|error| fail(error.to_string()))
	}) {
		message.push(':');

		for step in &plan.steps {
			let _ = write!(message, "\n  {step}");
		}
	}

	Err(fail(message))
}

/// Writes the migration from the latest one in `directory` to the tables in
/// the code, as `<now>_<name>`, and lists it in the registry. Returns its
/// directory's name.
///
/// # Errors
///
/// When nothing changed, `name` is not a plain name, a rename needs a hint,
/// or the files cannot be written.
pub fn create(directory: &Path, name: &str, now: &str) -> Result<String, MigrationError> {
	if name.is_empty()
		|| !name
			.chars()
			.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
	{
		return Err(fail(
			"a migration's name is lowercase letters, digits and `_`",
		));
	}

	let previous = latest(directory)?;
	let target = target()?;
	let plan = leptoskit_db_schema::diff(&previous, &target, &mut RequireHints)
		.map_err(|error| fail(error.to_string()))?;

	if plan.is_empty() {
		return Err(fail(
			"the tables haven't changed since the latest migration",
		));
	}

	let forward = leptoskit_db_schema::render(&plan, &previous, &target);
	let back = leptoskit_db_schema::diff(&target, &previous, &mut NoRenames)
		.map_err(|error| fail(error.to_string()))?;
	let reverse = leptoskit_db_schema::render(&back, &target, &previous);
	let full = format!("{now}_{name}");
	let migration = directory.join(&full);

	if migration.exists() {
		return Err(fail(format!("{} already exists", migration.display())));
	}

	let write = |file: &str, text: String| {
		fs::write(migration.join(file), text)
			.map_err(|error| fail(format!("{}: {error}", migration.join(file).display())))
	};

	fs::create_dir_all(&migration).map_err(|error| fail(error.to_string()))?;
	write("migration.sql", forward.sql())?;
	write("down.sql", reverse.sql())?;
	write("schema.json", target.to_json())?;

	let registry = directory.join("registry.txt");
	let mut text = fs::read_to_string(&registry).unwrap_or_default();

	if !text.contains("# The migrations") {
		text.insert_str(
			0,
			"# The migrations, in the order they apply. `reapers-migration` adds to it.\n",
		);
	}

	if !text.ends_with('\n') {
		text.push('\n');
	}

	text.push_str(&full);
	text.push('\n');
	fs::write(&registry, text).map_err(|error| fail(error.to_string()))?;

	Ok(full)
}
