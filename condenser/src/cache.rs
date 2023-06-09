//! Contains code ralated to parsing and saving cache data.

use serde::{Deserialize, Serialize};

use sustainity_collecting::errors::IoOrSerdeError;

/// Cached data from search over Wikidata data.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Wikidata {
    /// Manufacturer IDs.
    pub manufacturer_ids: Vec<sustainity_wikidata::data::StrId>,

    /// All entry classes.
    pub classes: Vec<sustainity_wikidata::data::StrId>,
}

/// Reads in the cache data.
pub fn load(path: &std::path::Path) -> Result<Wikidata, IoOrSerdeError> {
    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        let cache = serde_json::from_str(&contents)?;
        Ok(cache)
    } else {
        Ok(Wikidata::default())
    }
}
