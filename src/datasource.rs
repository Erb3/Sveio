use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct City {
	pub(crate) name: String,
	pub(crate) country: String,
	pub(crate) latitude: f64,
	pub(crate) longitude: f64,
}

impl City {
	pub(crate) fn anonymize(self) -> AnonymizedCity {
		AnonymizedCity {
			country: self.country,
			name: self.name,
		}
	}
}

#[derive(Serialize, Clone)]
pub(crate) struct AnonymizedCity {
	pub(crate) country: String,
	pub(crate) name: String,
}

pub(crate) struct Datasource {
	pub(crate) cities: Vec<City>,
}

impl Datasource {
	pub(crate) async fn new() -> Datasource {
		let mut cities: Vec<City> = serde_json::from_str(include_str!("../cities.json"))
			.expect("cities.json does not have correct format.");

		cities.shuffle(&mut thread_rng());

		Datasource { cities }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn can_load_cities() {
		assert_ne!(Datasource::new().await.cities.len(), 0);
	}

	#[tokio::test]
	async fn cities_are_randomized() {
		let datasource1 = Datasource::new().await;
		let datasource2 = Datasource::new().await;

		// Check if both the first and second city in the datasources matches
		// If it does, this suggests that the cities are not randomized
		assert_ne!(
			datasource1.cities.get(0).unwrap().name,
			datasource2.cities.get(0).unwrap().name
		);
		assert_ne!(
			datasource1.cities.get(1).unwrap().latitude,
			datasource2.cities.get(1).unwrap().latitude
		)
	}

	#[tokio::test]
	async fn can_anonymize_city() {
		let datasource = Datasource::new().await;
		let city = datasource.cities.get(0).unwrap();
		let anonymized = city.clone().anonymize();

		assert_eq!(city.name, anonymized.name);
		assert_eq!(city.country, anonymized.country);
	}
}
