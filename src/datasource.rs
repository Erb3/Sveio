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

#[derive(Serialize)]
pub struct AnonymizedCity<'a> {
	pub country: &'a str,
	pub name: &'a str,
}

pub struct Datasource {
	pub cities: Vec<City>,
	length: usize,
	index: usize,
}

impl Datasource {
	pub fn get_next(&mut self) -> &City {
		let city: &City = self.cities.get(self.index).unwrap();

		self.index += 1;
		if self.index == self.length - 1 {
			self.index = 0;
		}

		return city;
	}
}

pub async fn new() -> Datasource {
	let mut cities: Vec<City> = serde_json::from_str(include_str!("../cities.json"))
		.expect("cities.json does not have correct format.");

	cities.shuffle(&mut thread_rng());

	Datasource {
		cities: cities.clone(),
		length: cities.len(),
		index: 0,
	}
}
