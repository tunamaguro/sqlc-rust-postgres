use std::{
    backtrace::{Backtrace, BacktraceStatus},
    fmt::Display,
};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidRustType(String),
    MissingColInfo(String),
    UnSupportedAnnotation(String),
    AnyError(String),
    Decode(prost::DecodeError),
    BackTrace {
        source: Box<Self>,
        backtrace: Backtrace,
    },
}

impl Error {
    fn into_backtrace(self) -> Self {
        let backtrace = Backtrace::force_capture();
        match backtrace.status() {
            BacktraceStatus::Captured => Self::BackTrace {
                source: Box::new(self),
                backtrace,
            },
            _ => self,
        }
    }

    pub(crate) fn invalid_rust_type<S: Display>(rs_type: S) -> Self {
        Self::InvalidRustType(rs_type.to_string()).into_backtrace()
    }

    pub(crate) fn missing_col_info<S: Display>(col_name: S) -> Self {
        Self::MissingColInfo(col_name.to_string()).into_backtrace()
    }

    pub(crate) fn db_type_cannot_map<S: Display>(db_type: S) -> Self {
        Self::InvalidRustType(db_type.to_string()).into_backtrace()
    }

    pub(crate) fn unsupported_annotation<S: Display>(annotation: S) -> Self {
        Self::UnSupportedAnnotation(annotation.to_string()).into_backtrace()
    }

    pub(crate) fn any_error<S: Display>(message: S) -> Self {
        Self::AnyError(message.to_string()).into_backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => error.fmt(f),
            Error::InvalidRustType(db_type) => write!(
                f,
                "Cannot find rust type that matches column type of `{}`. Add an entry to the 'overrides' section in your sqlc.json configuration.",
                db_type
            ),
            Error::MissingColInfo(col_name) => {
                write!(f, "no type information for column {}", col_name)
            }
            Error::UnSupportedAnnotation(annotation) => {
                write!(f, "query annotation `{}` is not supported", annotation)
            }
            Error::Decode(e) => e.fmt(f),
            Error::AnyError(message) => {
                const ISSUE_URL: &str =
                    "https://github.com/tunamaguro/sqlc-rust-postgres/issues/new";
                write!(
                    f,
                    "It looks like you've encountered an unexpected bug. Please consider reporting this issue at {} so we can investigate further.\nDetail: {}",
                    ISSUE_URL, message
                )
            }
            Error::BackTrace { source, backtrace } => {
                write!(f, "{}\n{}", source, backtrace)
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value).into_backtrace()
    }
}

impl From<prost::DecodeError> for Error {
    fn from(value: prost::DecodeError) -> Self {
        Self::Decode(value).into_backtrace()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(error) => Some(error),
            Error::BackTrace { source, .. } => Some(source),
            _ => None,
        }
    }
}
