pub(crate) fn calculate_score(kms: f64) -> u64 {
	let kilometers = kms as u64;
	if kilometers > 1500 {
		return 0;
	}

	(1500 - kilometers).div_ceil(4)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn can_calculate_score() {
		assert_eq!(calculate_score(1500.0), 0);
		assert_eq!(calculate_score(1501.0), 0);
		assert_eq!(calculate_score(1504.0), 0);
		assert_eq!(calculate_score(12893.12), 0);
		assert_eq!(calculate_score(4.1), 374);
		assert_eq!(calculate_score(3.9), 375);
		assert_eq!(calculate_score(1.0), 375);
		assert_eq!(calculate_score(0.99), 375);
		assert_eq!(calculate_score(0.2), 375);
		assert_eq!(calculate_score(0.0), 375);
	}
}
