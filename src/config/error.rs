use thiserror::Error;

#[derive(Debug, Error)]
#[error("error when parse var '{var}': {kind}")]
pub struct VarsBuildError {
    pub var: String,
    pub kind: VarsParseError,
}

#[derive(Debug, Error)]
pub enum VarsParseError {
    #[error("unclosed brace at position {0}")]
    UnclosedBrace(usize),

    #[error("empty variable expression found at position {0}")]
    EmptyVarName(usize),

    #[error("unknown variable '{0}' found at {1}")]
    UnknowndVar(String, usize),
}

#[derive(Debug, Error)]
pub enum PkgsParseError {
    #[error(transparent)]
    VarsBuild(#[from] VarsBuildError),

    #[error(transparent)]
    VarsParse(#[from] VarsParseError),

    #[error("could not get filename from path '{0}'")]
    NoneFilename(String),
}
