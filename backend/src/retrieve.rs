use std::collections::HashMap;

use sustainity_api::models as api;

use crate::{db::Db, errors::BackendError, models::SearchResultVariant};

#[derive(Clone, Debug, PartialEq)]
struct ScoredResult {
    score: f64,
    result: api::TextSearchResult,
}

impl ScoredResult {
    pub fn with_added_score(&mut self, score: f64) {
        self.score += score;
    }
}

#[derive(Clone, Debug, Default)]
struct ResultCollector {
    results: HashMap<String, ScoredResult>,
}

impl ResultCollector {
    // Adds results by giving them some score.
    //
    // The score is better if:
    // - the matched keyword is closer to the beginning of the query
    // - the matched keyword constitutes the longer part of the whole label
    pub fn add(&mut self, results: &[api::TextSearchResult], matching: &str, index: Option<usize>) {
        let index_score = if let Some(index) = index { 1.0 / (index + 1) as f64 } else { 10.0 };

        for result in results {
            let item_score = matching.len() as f64 / result.label.len() as f64;
            let total_score = 1.0 + index_score + item_score;

            self.results
                .entry(result.id.clone())
                .and_modify(|e| e.with_added_score(total_score))
                .or_insert_with(|| ScoredResult { score: total_score, result: result.clone() });
        }
    }

    pub fn gather_scored_results(self) -> Vec<ScoredResult> {
        use std::cmp::Ordering;

        let mut results: Vec<ScoredResult> = self.results.into_values().collect();
        results.sort_by(|a, b| match PartialOrd::partial_cmp(&b.score, &a.score) {
            None | Some(Ordering::Equal) => Ord::cmp(&a.result.id, &b.result.id),
            Some(ordering) => ordering,
        });
        results
    }

    pub fn gather_results(self) -> Vec<api::TextSearchResult> {
        self.gather_scored_results().into_iter().map(|r| r.result).collect()
    }
}

pub async fn library_contents(db: &Db) -> Result<Vec<api::LibraryItemShort>, BackendError> {
    Ok(db
        .get_library_contents()
        .await?
        .into_iter()
        .filter_map(|i| i.try_into_api_short().ok())
        .collect())
}

pub async fn library_item(
    topic: api::LibraryTopic,
    db: &Db,
) -> Result<Option<api::LibraryItemFull>, BackendError> {
    let topic_name = topic.to_string();
    if let Some(item) = db.get_library_item(&topic_name).await? {
        let presentation = db.get_presentation(&topic_name).await?.map(|p| p.into_api());
        let item = item.try_into_api_full(presentation)?;
        Ok(Some(item))
    } else {
        Ok(None)
    }
}

pub async fn organisation(
    id: &str,
    db: &Db,
) -> Result<Option<api::OrganisationFull>, BackendError> {
    if let Some(org) = db.get_organisation(id).await? {
        let products = db
            .find_organisation_products(id)
            .await?
            .into_iter()
            .map(|p| p.into_api_short())
            .collect();
        let org = org.into_api_full(products);
        Ok(Some(org))
    } else {
        Ok(None)
    }
}

pub async fn product(
    id: &str,
    region: Option<&str>,
    db: &Db,
) -> Result<Option<api::ProductFull>, BackendError> {
    if let Some(prod) = db.get_product(id).await? {
        let manufacturers = db
            .find_product_manufacturers(id)
            .await?
            .into_iter()
            .map(|m| m.into_api_short())
            .collect();
        let alternatives = product_alternatives(id, region, db).await?;
        let prod = prod.into_api_full(manufacturers, alternatives);
        Ok(Some(prod))
    } else {
        Ok(None)
    }
}

pub async fn product_alternatives(
    id: &str,
    region_code: Option<&str>,
    db: &Db,
) -> Result<Vec<api::CategoryAlternatives>, BackendError> {
    let mut result = Vec::new();
    let categories = db.find_product_categories(id).await?;
    for category in categories {
        let alternatives = db
            .find_product_alternatives(id, &category, region_code)
            .await?
            .into_iter()
            .map(|a| a.into_api_short())
            .collect();
        result.push(api::CategoryAlternatives { category, alternatives });
    }
    Ok(result)
}

pub async fn search_by_text(
    query: String,
    db: &Db,
) -> Result<Vec<api::TextSearchResult>, BackendError> {
    let mut collector = ResultCollector::default();
    let mut matches: Vec<&str> = query.split(' ').collect();
    matches.retain(|m| !m.is_empty());

    if matches.len() == 1 {
        let lowercase_match = matches.first().unwrap().to_lowercase();
        let uppercase_match = matches.first().unwrap().to_uppercase();

        // Search organisation by VAT
        {
            let items = db.search_organisations_substring_by_vat_number(&uppercase_match).await?;
            let items = SearchResultVariant::Organisation.convert(items);
            collector.add(&items, &uppercase_match, None);
        }

        // Search product by GTIN
        if lowercase_match.len() < 15 {
            let gtin = format!("{lowercase_match:0>14}");
            let items = db.search_products_exact_by_gtin(&gtin).await?;
            let items = SearchResultVariant::Product.convert(items);
            collector.add(&items, &lowercase_match, None);
        }

        // Search organisation by website
        {
            let items = db.search_organisations_substring_by_website(&lowercase_match).await?;
            let items = SearchResultVariant::Organisation.convert(items);
            collector.add(&items, &lowercase_match, None);
        }
    }

    // Search organisations and products by keyword
    let lowercase_matches: Vec<String> = matches.into_iter().map(|m| m.to_lowercase()).collect();
    for (i, m) in lowercase_matches.iter().enumerate() {
        let items = db.search_organisations_exact_by_keyword(m).await?;
        let items = SearchResultVariant::Organisation.convert(items);
        collector.add(&items, m, Some(i));
    }
    for (i, m) in lowercase_matches.iter().enumerate() {
        let items = db.search_products_exact_by_keyword(m).await?;
        let items = SearchResultVariant::Product.convert(items);
        collector.add(&items, m, Some(i));
    }

    Ok(collector.gather_results())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sustainity_api::models as api;

    fn prepare_data() -> (api::TextSearchResult, api::TextSearchResult, api::TextSearchResult) {
        let r1 = api::TextSearchResult {
            variant: api::TextSearchResultVariant::Product,
            label: "Fairphone 4".into(),
            id: "1".into(),
        };

        let r2 = api::TextSearchResult {
            variant: api::TextSearchResultVariant::Product,
            label: "Samsung 4".into(),
            id: "2".into(),
        };

        let r3 = api::TextSearchResult {
            variant: api::TextSearchResultVariant::Product,
            label: "Fairphone 3".into(),
            id: "3".into(),
        };

        (r1, r2, r3)
    }

    /// No sorting hints are given:
    /// - the most repeated item is the first
    /// - ties are proken by sorting by the label
    #[test]
    fn simple() {
        let (r1, r2, r3) = prepare_data();

        let s1 = ScoredResult { result: r1.clone(), score: (1.0 + 10.0) + (1.0 + 10.0) };
        let s2 = ScoredResult { result: r2.clone(), score: (1.0 + 10.0) };
        let s3 = ScoredResult { result: r3.clone(), score: (1.0 + 10.0) };

        let expected_results = [s1, s2, s3];

        {
            let mut collector = ResultCollector::default();
            collector.add(&[r2.clone(), r1.clone()], "", None);
            collector.add(&[r3.clone(), r1.clone()], "", None);

            assert_eq!(collector.gather_scored_results(), expected_results);
        }
        {
            let mut collector = ResultCollector::default();
            collector.add(&[r1.clone(), r3.clone()], "", None);
            collector.add(&[r1.clone(), r2.clone()], "", None);

            assert_eq!(collector.gather_scored_results(), expected_results);
        }
    }

    /// Only position in the query given as a sorting hint.
    /// - the phrase more in the front of the query is given a boost
    #[test]
    fn index() {
        let (r1, r2, r3) = prepare_data();

        let s1 = ScoredResult { result: r1.clone(), score: (1.0 + 1.0) + (1.0 + 0.5) };
        let s2 = ScoredResult { result: r2.clone(), score: (1.0 + 0.5) };
        let s3 = ScoredResult { result: r3.clone(), score: (1.0 + 1.0) };

        let expected_results = [s1, s3, s2];

        let mut collector = ResultCollector::default();
        collector.add(&[r2.clone(), r1.clone()], "", Some(1));
        collector.add(&[r3.clone(), r1.clone()], "", Some(0));

        assert_eq!(collector.gather_scored_results(), expected_results);
    }

    /// Only the matched phrase given as a sorting hint.
    /// - the phrase that constitutes a bigger chunk of the whole label is given a boost
    #[test]
    fn importance() {
        let (r1, r2, r3) = prepare_data();

        let s1 =
            ScoredResult { result: r1.clone(), score: (11.0 + 9.0 / 11.0) + (11.0 + 1.0 / 11.0) };
        let s2 = ScoredResult { result: r2.clone(), score: (11.0 + 1.0 / 9.0) };
        let s3 = ScoredResult { result: r3.clone(), score: (11.0 + 9.0 / 11.0) };

        let expected_results = [s1, s3, s2];

        let mut collector = ResultCollector::default();
        collector.add(&[r2.clone(), r1.clone()], "4", None);
        collector.add(&[r3.clone(), r1.clone()], "Fairphone", None);

        assert_eq!(collector.gather_scored_results(), expected_results);
    }
}
