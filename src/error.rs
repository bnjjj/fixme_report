use std::io;

use config_rs::ConfigError;
use github_rs::errors::Error as GhError;
use handlebars::TemplateRenderError;
use thiserror::Error;
use unidiff::Error as UnidiffError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("git diff/patch parser error : {0:?}")]
    ParseError(#[from] UnidiffError),

    #[error("configuration error : {0:?}")]
    ConfigError(#[from] ConfigError),

    #[error("unknown configuration type : {0:?}")]
    UnknownConfigType(String),

    #[error("GitHub error : {0:?}")]
    GithubError(#[from] GhError),

    #[error("Template error on comment : {0:?}")]
    TemplateError(#[from] TemplateRenderError),

    #[error("IO error on comment : {0:?}")]
    IoError(#[from] io::Error),

    #[error("HTTP error : {status_code} - {body}")]
    HttpError { status_code: u16, body: String },
}
