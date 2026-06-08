use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error in {file}: {source}")]
    Toml {
        file: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("spec error: {0}")]
    Spec(String),

    #[error("check failed: {0}")]
    Check(String),

    #[error("XML parse error: {0}")]
    Xml(String),
}
