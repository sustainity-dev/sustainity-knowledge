use crate::{advisors, config, errors, knowledge};

fn convert_info(
    info: &sustainity_collecting::sustainity::data::LibraryInfo,
) -> knowledge::LibraryInfo {
    knowledge::LibraryInfo {
        id: info.id.clone(),
        title: info.title.clone(),
        article: info.article.clone(),
    }
}

pub struct Transcriptor;

impl Transcriptor {
    pub fn transcribe(config: &config::TranscriptionConfig) -> Result<(), errors::ProcessingError> {
        let sustainity = advisors::SustainityAdvisor::load(&config.library_source_path)?;
        let library: Vec<knowledge::LibraryInfo> =
            sustainity.get_info().iter().map(convert_info).collect();
        let contents = serde_json::to_string_pretty(&library)?;
        std::fs::write(&config.library_target_path, contents)?;
        Ok(())
    }
}
