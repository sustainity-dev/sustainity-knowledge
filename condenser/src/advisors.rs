//! Contains code ralated to parsing source data.

use std::collections::HashSet;

use consumers_collecting::{bcorp, errors::IoOrParsingError, tco};

use crate::utils;

/// Holds the information read from the `BCorp` data.
pub struct BCorpAdvisor {
    /// Domains of `BCorp` companies.
    domains: HashSet<String>,
}

impl BCorpAdvisor {
    /// Constructs a new `BCorpAdvisor`.
    pub fn new(records: &[bcorp::data::Record]) -> Self {
        let domains: HashSet<String> =
            records.iter().map(|r| utils::extract_domain_from_url(&r.website)).collect();
        Self { domains }
    }

    /// Loads a new `BCorpAdvisor` from a file.
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let data = consumers_collecting::bcorp::reader::parse(&path)?;
        Ok(Self::new(&data))
    }

    /// Checks if at least one of the passed domains corresponds to a `BCorp` company.
    pub fn has_domains(&self, domains: &HashSet<String>) -> bool {
        for domain in domains {
            if self.domains.contains(domain) {
                return true;
            }
        }
        false
    }
}

/// Holds the information read from the `BCorp` data.
pub struct TcoAdvisor {
    /// Wikidata IDs of companies certifies by TCO.
    companies: HashSet<String>,
}

impl TcoAdvisor {
    /// Constructs a new `TcoAdvisor`.
    pub fn new(entries: &[tco::data::Entry]) -> Self {
        Self { companies: entries.iter().map(|entry| entry.wikidata_id.clone()).collect() }
    }

    /// Loads a new `Tcodvisor` from a file.
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, IoOrParsingError> {
        let data = consumers_collecting::tco::reader::parse(&path)?;
        Ok(Self::new(&data))
    }

    /// Checks if the comapny was certified.
    pub fn has_company(&self, company_id: &String) -> bool {
        self.companies.contains(company_id)
    }
}

/// Holds the information read from out internal data set.
pub struct ConsumersAdvisor {}

impl ConsumersAdvisor {
    /// Constructs a new `ConsumersAdvisor`.
    pub fn new() -> Self {
        Self {}
    }

    /// Loads a new `ConsumersAdvisor` from a file.
    pub fn load<P: AsRef<std::path::Path>>(_path: P) -> Self {
        ConsumersAdvisor::new()
    }
}
