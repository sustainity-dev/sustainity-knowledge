//! Contains code ralated to parsing source data.

use crate::{advisors, cache, config};

/// Holds all the source data.
pub struct Sources {
    /// Wikidata cache.
    pub cache: cache::WikidataCache,

    /// BCorp data.
    pub bcorp: advisors::BCorpAdvisor,

    /// Consumers data.
    pub consumers: advisors::ConsumersAdvisor,
}

impl Sources {
    /// Constructs a new `Sources`.
    pub fn new(config: config::Config) -> Result<Self, ()> {
        let cache = cache::CacheReader::new(config.clone()).read().unwrap();

        let bcorp_data = consumers_collecting::bcorp::reader::parse(&config.bcorp_path).unwrap();
        let bcorp_advisor = advisors::BCorpAdvisor::new(&bcorp_data);

        let consumers_advisor = advisors::ConsumersAdvisor::load(&config.consumers_path).unwrap();

        Ok(Self { cache: cache, bcorp: bcorp_advisor, consumers: consumers_advisor })
    }
}
