//! The one source of chance in the rules.

/// A small deterministic generator (mulberry32).
///
/// Everything a reaper's shuffle, a husk's path or a rift's schedule derives
/// from a seed goes through this type, so the game and the server always
/// agree. No platform generator is used because its sequence is not
/// guaranteed to stay the same between releases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeededRandom {
	state: u32,
}

impl SeededRandom {
	/// A generator seeded with the low 32 bits of [`seed`](Self).
	pub fn new(seed: i64) -> Self {
		Self { state: seed as u32 }
	}

	/// The next value in `[0, 2^32)`.
	#[allow(
		clippy::decimal_bitwise_operands,
		reason = "mulberry32's own constants"
	)]
	pub fn next_u32(&mut self) -> u32 {
		self.state = self.state.wrapping_add(0x6D2B_79F5);

		let mut t = self.state;
		t = (t ^ (t >> 15)).wrapping_mul(t | 1);
		t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));

		t ^ (t >> 14)
	}

	/// The next value in `[0, 1)`.
	pub fn next_double(&mut self) -> f64 {
		f64::from(self.next_u32()) / 4_294_967_296.0
	}

	/// The next integer in `[0, max)`. A `max` of zero or less gives zero.
	pub fn next_int(&mut self, max: i64) -> i64 {
		if max <= 0 {
			return 0;
		}

		crate::math::floor_int(self.next_double() * max as f64)
	}

	/// The next index into a list of `len` items.
	pub fn next_index(&mut self, len: usize) -> usize {
		self.next_int(len as i64) as usize
	}

	/// The next value in `[min, max)`.
	pub fn next_range(&mut self, min: f64, max: f64) -> f64 {
		min + self.next_double() * (max - min)
	}

	/// True with probability `chance`.
	pub fn next_chance(&mut self, chance: f64) -> bool {
		self.next_double() < chance
	}

	/// Picks one of `options` uniformly. Panics on an empty slice, which the
	/// rules never pass.
	pub fn pick<T: Copy>(&mut self, options: &[T]) -> T {
		options[self.next_index(options.len())]
	}

	/// Picks an index from `weights`, each chosen in proportion to its
	/// weight.
	pub fn pick_weighted(&mut self, weights: &[f64]) -> usize {
		let total = weights.iter().fold(0.0, |sum, weight| sum + weight);
		let mut roll = self.next_double() * total;

		for (index, weight) in weights.iter().enumerate() {
			roll -= weight;

			if roll < 0.0 {
				return index;
			}
		}

		weights.len().saturating_sub(1)
	}
}

/// Mixes `values` into one 32-bit seed (FNV-1a over each value's low four
/// bytes).
///
/// Used wherever a seed is derived from identifiers, such as the seed of the
/// hunt for one husk, so the derivation is stable across platforms.
pub fn mix_seed(values: &[i64]) -> i64 {
	let mut hash: u32 = 0x811C_9DC5;

	for &value in values {
		for shift in (0..32).step_by(8) {
			hash ^= ((value >> shift) & 0xFF) as u32;
			hash = hash.wrapping_mul(0x0100_0193);
		}
	}

	i64::from(hash)
}

/// A stable 32-bit hash of `text`, over its UTF-16 code units, for seeding
/// from string identifiers.
pub fn hash_text(text: &str) -> i64 {
	let units: Vec<i64> = text.encode_utf16().map(i64::from).collect();

	mix_seed(&units)
}
