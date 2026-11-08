//! How the Dart rules read numbers out of text, kept so that a record they
//! accepted is accepted here and one they refused is refused.

/// Dart's `int.tryParse` without a radix: surrounding whitespace, an
/// optional sign, then decimal digits or `0x` and hexadecimal digits. `None`
/// for anything else, or for a value that does not fit in 64 bits.
pub fn parse_int(text: &str) -> Option<i64> {
	let trimmed = text.trim();
	let (negative, unsigned) = match trimmed.as_bytes().first() {
		Some(b'-') => (true, &trimmed[1..]),
		Some(b'+') => (false, &trimmed[1..]),
		_ => (false, trimmed),
	};
	let (radix, digits) = match unsigned
		.strip_prefix("0x")
		.or_else(|| unsigned.strip_prefix("0X"))
	{
		Some(hex) => (16, hex),
		None => (10, unsigned),
	};

	if digits.is_empty() || !digits.chars().all(|digit| digit.is_digit(radix)) {
		return None;
	}

	let magnitude = u64::from_str_radix(digits, radix).ok()?;

	if negative {
		0_i64.checked_sub_unsigned(magnitude)
	} else if radix == 16 {
		// Dart reads hexadecimal up to 2^64 - 1 as the 64-bit pattern.
		Some(magnitude as i64)
	} else {
		i64::try_from(magnitude).ok()
	}
}
