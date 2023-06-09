use thiserror::Error;

pub use sustainity_collecting::errors::IoOrSerdeError;
pub use sustainity_wikidata::dump::LoaderError;
pub use sustainity_wikidata::errors::ParseIdError;

/// Error returned if config checking failed.
#[derive(Error, Debug)]
pub enum ConfigCheckError {
    #[error("Path '{0}' does not exist")]
    PathDoesNotExist(std::path::PathBuf),

    #[error("Path '{0}' is not a file")]
    PathIsNotAFile(std::path::PathBuf),

    #[error("Path '{0}' is not a directory")]
    PathIsNotADir(std::path::PathBuf),

    #[error("Path '{0}' already exists")]
    PathAlreadyExists(std::path::PathBuf),

    #[error("Base of '{0}' does not exist")]
    BaseDoesNotExist(std::path::PathBuf),

    #[error("Base of '{0}' is not a directory")]
    BaseIsNotADirectory(std::path::PathBuf),
}

/// Error related to validating the input data.
#[derive(Error, Debug)]
pub enum SourcesCheckError {
    /// IDs were duplicated while expected to be unique.
    #[error("Repeated IDs: {0:?}")]
    RepeatedIds(std::collections::HashSet<sustainity_wikidata::data::Id>),
}

/// Error returned when a problem with processing.
#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("IO error: {0}")]
    Io(std::io::Error),

    #[error("Csv parsing error: {0}")]
    Csv(csv::Error),

    #[error("parsing error: {0}")]
    Json(serde_json::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(serde_yaml::Error),

    #[error("Variant parsing error: {0}")]
    Variant(serde_variant::UnsupportedType),

    #[error("Task joining error: {0}")]
    Join(tokio::task::JoinError),

    #[error("Unknown compression method")]
    CompressionMethod,

    #[error("Channel sending error: {0}")]
    Channel(async_channel::SendError<std::string::String>),

    #[error("Config check: {0}")]
    ConfigCheck(ConfigCheckError),

    #[error("Sources check: {0}")]
    SourcesCheck(SourcesCheckError),

    #[error("ID parsing: {0}")]
    IdParsing(ParseIdError),

    #[error("Mutex lock")]
    MutexLock,
}

impl From<std::io::Error> for ProcessingError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<csv::Error> for ProcessingError {
    fn from(error: csv::Error) -> Self {
        Self::Csv(error)
    }
}

impl From<serde_json::Error> for ProcessingError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<serde_yaml::Error> for ProcessingError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::Yaml(error)
    }
}

impl From<serde_variant::UnsupportedType> for ProcessingError {
    fn from(error: serde_variant::UnsupportedType) -> Self {
        Self::Variant(error)
    }
}

impl From<tokio::task::JoinError> for ProcessingError {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::Join(error)
    }
}

impl From<async_channel::SendError<std::string::String>> for ProcessingError {
    fn from(error: async_channel::SendError<std::string::String>) -> Self {
        Self::Channel(error)
    }
}

impl<T> From<std::sync::PoisonError<T>> for ProcessingError {
    fn from(_error: std::sync::PoisonError<T>) -> Self {
        Self::MutexLock
    }
}

impl From<ConfigCheckError> for ProcessingError {
    fn from(error: ConfigCheckError) -> Self {
        Self::ConfigCheck(error)
    }
}

impl From<SourcesCheckError> for ProcessingError {
    fn from(error: SourcesCheckError) -> Self {
        Self::SourcesCheck(error)
    }
}

impl From<ParseIdError> for ProcessingError {
    fn from(error: ParseIdError) -> Self {
        Self::IdParsing(error)
    }
}

impl From<IoOrSerdeError> for ProcessingError {
    fn from(error: IoOrSerdeError) -> Self {
        match error {
            IoOrSerdeError::Io(error) => Self::Io(error),
            IoOrSerdeError::Csv(error) => Self::Csv(error),
            IoOrSerdeError::Json(error) => Self::Json(error),
            IoOrSerdeError::Yaml(error) => Self::Yaml(error),
        }
    }
}

impl From<LoaderError> for ProcessingError {
    fn from(error: LoaderError) -> Self {
        match error {
            LoaderError::Io(error) => Self::Io(error),
            LoaderError::CompressionMethod => Self::CompressionMethod,
        }
    }
}
