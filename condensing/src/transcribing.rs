use sustainity_models::write as models;

use crate::{advisors, config, errors};

pub struct Transcriptor;

impl Transcriptor {
    /// Rust the transcription command.
    ///
    /// # Errors
    ///
    /// Returns `Err` if reading, parsing or saving required data failed.
    pub fn transcribe(config: &config::TranscriptionConfig) -> Result<(), errors::ProcessingError> {
        let sustainity = advisors::SustainityLibraryAdvisor::load(&config.library_file_path)?;
        let mut library = Vec::<models::LibraryItem>::new();
        for info in sustainity.get_info() {
            let id: &str = serde_variant::to_variant_name(&info.id)?;
            let article_path = config.library_dir_path.join(id).with_extension("md");
            crate::utils::path_exists(&article_path)?;

            let article = std::fs::read_to_string(&article_path)?;
            library.push(models::LibraryItem {
                id: info.id.to_str().into(),
                title: info.title.clone(),
                summary: info.summary.clone(),
                article,
            });
        }
        serde_jsonlines::write_json_lines(&config.library_target_path, library)?;
        Ok(())
    }
}
