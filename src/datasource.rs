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

pub async fn get_cities() -> Vec<City> {
	serde_json::from_str(include_str!("../cities.json"))
		.expect("cities.json does not have correct format.")
}
