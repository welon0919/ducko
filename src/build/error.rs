use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Folder content not found")]
    ContentNotFound,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Front matter not found")]
    FrontMatterNotFound,
    #[error("Front matter not closed")]
    FrontMatterNotClosed,
    #[error(transparent)]
    MetadataFormatError(#[from] serde_yaml::Error),
    #[error("Error building file {0}: {1}")]
    ErrorBuildingFile(String, Box<BuildError>),
    #[error(transparent)]
    TemplateError(#[from] tera::Error),
    #[error(transparent)]
    CopyError(#[from] fs_extra::error::Error),
}
