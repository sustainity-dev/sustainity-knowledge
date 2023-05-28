//! Contains code ralated to parsing source data.

use std::collections::{HashMap, HashSet};

use consumers_collecting::{
    bcorp, consumers, errors::IoOrSerdeError, fashion_transparency_index, tco,
};

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
    pub fn load<P>(path: P) -> Result<Self, IoOrSerdeError>
    where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        if utils::is_path_ok(path.as_ref()) {
            let data = consumers_collecting::bcorp::reader::parse(&path)?;
            Ok(Self::new(&data))
        } else {
            log::warn!("Could not access {path:?}. BCorp data won't be loaded!");
            Ok(Self::new(&[]))
        }
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
    companies: HashSet<consumers_wikidata::data::Id>,
}

impl TcoAdvisor {
    /// Constructs a new `TcoAdvisor`.
    pub fn new(entries: &[tco::data::Entry]) -> Self {
        Self { companies: entries.iter().map(|entry| entry.wikidata_id.clone()).collect() }
    }

    /// Loads a new `Tcodvisor` from a file.
    pub fn load<P>(path: P) -> Result<Self, IoOrSerdeError>
    where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        if utils::is_path_ok(path.as_ref()) {
            let data = consumers_collecting::tco::reader::parse(&path)?;
            Ok(Self::new(&data))
        } else {
            log::warn!("Could not access {path:?}. TCO data won't be loaded!");
            Ok(Self::new(&[]))
        }
    }

    /// Checks if the company was certified.
    pub fn has_company(&self, company_id: &consumers_wikidata::data::Id) -> bool {
        self.companies.contains(company_id)
    }
}

/// Holds the information read from the `Fashion Transparency Index` data.
pub struct FashionTransparencyIndexAdvisor {
    /// Wikidata IDs of companies certifies by TCO.
    entries: HashMap<consumers_wikidata::data::Id, usize>,
}

impl FashionTransparencyIndexAdvisor {
    /// Constructs a new `TcoAdvisor`.
    pub fn new(entries: &[fashion_transparency_index::data::Entry]) -> Self {
        Self {
            entries: entries.iter().map(|entry| (entry.wikidata_id.clone(), entry.score)).collect(),
        }
    }

    /// Loads a new `Tcodvisor` from a file.
    pub fn load<P>(path: P) -> Result<Self, IoOrSerdeError>
    where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        if utils::is_path_ok(path.as_ref()) {
            let data = fashion_transparency_index::reader::parse(&path)?;
            Ok(Self::new(&data))
        } else {
            log::warn!(
                "Could not access {path:?}. Fashion Transparency Index data won't be loaded!"
            );
            Ok(Self::new(&[]))
        }
    }

    /// Get the score for the given company.
    pub fn get_score(&self, company_id: &consumers_wikidata::data::Id) -> Option<usize> {
        self.entries.get(company_id).copied()
    }
}

/// Holds the information read from out internal data set.
pub struct ConsumersAdvisor {
    /// Topic info.
    info: Vec<consumers::data::Info>,
}

impl ConsumersAdvisor {
    /// Constructs a new `ConsumersAdvisor`.
    pub fn new(info: Vec<consumers::data::Info>) -> Self {
        Self { info }
    }

    /// Loads a new `ConsumersAdvisor` from a file.
    pub fn load<P>(path: P) -> Result<Self, IoOrSerdeError>
    where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        if utils::is_path_ok(path.as_ref()) {
            let data = consumers_collecting::consumers::reader::parse(&path)?;
            Ok(Self::new(data))
        } else {
            log::warn!("Could not access {path:?}. Consumers data won't be loaded!");
            Ok(Self::new(Vec::new()))
        }
    }

    /// Returns all info.
    pub fn get_info(&self) -> &[consumers::data::Info] {
        &self.info
    }
}
