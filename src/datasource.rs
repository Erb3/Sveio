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

pub async fn new() -> Datasource {
	let mut cities: Vec<City> = serde_json::from_str(include_str!("../cities.json"))
		.expect("cities.json does not have correct format.");

	cities.shuffle(&mut thread_rng());

	Datasource { cities }
}
