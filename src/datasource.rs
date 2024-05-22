use serde::{Deserialize, Serialize};
use std::fs;

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

pub fn get_cities() -> Vec<City> {
    let cities_str = fs::read_to_string("./cities.json").expect("Unable to read cities.json file.");
    serde_json::from_str(&cities_str).expect("cities.json does not have correct format.")
}
