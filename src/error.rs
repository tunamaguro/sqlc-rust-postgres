#[derive(Debug, Clone)]
pub enum Error {
    InvalidIdent,
    InvalidRustType,
    NotFoundColType,
    NotFoundRustType,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidIdent => todo!(),
            Error::InvalidRustType => todo!(),
            Error::NotFoundColType => todo!(),
            Error::NotFoundRustType => todo!(),
        }
    }
}

impl std::error::Error for Error {}
