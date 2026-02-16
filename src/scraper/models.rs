use serde::{Deserialize, Serialize};

// prop
//  ├── source
//  │    ├── name
//  │    ├── id
//  │    └── listing_id
//  ├── location
//  │    ├── address
//  │    │    ├── line
//  │    │    ├── city
//  │    │    ├── state_code
//  │    │    ├── postal_code
//  │    │    └── country
//  │    ├── county
//  │    │    ├── name
//  │    │    └── fips_code
//  │    └── coordinate
//  │         ├── lat
//  │         └── lon
//  └── description
//       ├── beds
//       ├── baths
//       ├── lot_sqft
//       └── type

fn string_or_int<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => Ok(n.as_i64()),
        serde_json::Value::String(s) => s.parse::<i64>().map(Some).map_err(D::Error::custom),
        serde_json::Value::Null => Ok(None),
        _ => Err(D::Error::custom(
            "expected a number or string for fips_code",
        )),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Property {
    pub source: Source,
    pub location: Location,
    pub description: Option<Description>,

    pub status: Option<String>,
    pub list_price: Option<i64>,
    pub price_reduced: Option<i64>,
    pub sold_price: Option<i64>,
    pub flags: Option<Flags>,
    pub currency: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Source {
    pub name: Option<String>,
    pub id: Option<String>,
    #[serde(rename = "listing_id")]
    pub listing_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Location {
    pub address: Option<Address>,
    pub county: Option<County>,
    pub coordinate: Option<Coordinate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Address {
    pub line: Option<String>,
    pub city: Option<String>,
    #[serde(rename = "state_code")]
    pub state_code: Option<String>,
    #[serde(rename = "postal_code")]
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct County {
    pub name: Option<String>,
    #[serde(rename = "fips_code", deserialize_with = "string_or_int")]
    pub fips_code: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Coordinate {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Description {
    pub beds: Option<i64>,
    pub baths: Option<i64>,
    #[serde(rename = "lot_sqft")]
    pub lot_sqft: Option<i64>,
    #[serde(rename = "type")]
    pub property_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Flags {
    pub is_coming_soon: Option<bool>,
    pub is_contingent: Option<bool>,
    pub is_foreclosure: Option<bool>,
    pub is_new_construction: Option<bool>,
    pub is_new_listing: Option<bool>,
    pub is_pending: Option<bool>,

    // "is_price_reduced" appears both in flags and top-level JSON, optional
    pub is_price_reduced: Option<bool>,
}
