use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct City {
	pub name: String,
	pub country: String,
	pub latitude: f64,
	pub longitude: f64,
}

impl City {
	pub fn anonymize(self) -> AnonymizedCity {
		AnonymizedCity {
			country: self.country,
			name: self.name,
		}
	}
}

#[derive(Serialize)]
pub struct AnonymizedCity {
	pub country: String,
	pub name: String,
}

pub struct Datasource {
	pub cities: Vec<City>,
}

impl Datasource {
	pub async fn new() -> Datasource {
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
		assert!(
			datasource1.cities.get(0).unwrap().name != datasource2.cities.get(0).unwrap().name
				&& (datasource1.cities.get(1).unwrap().name
					!= datasource2.cities.get(1).unwrap().name)
		);
	}
}
