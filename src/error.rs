use std::{
    backtrace::{Backtrace, BacktraceStatus},
    panic::Location,
};

#[derive(Debug)]
pub struct Error {
    pub message: String,
    location: &'static Location<'static>,
    backtrace: Backtrace,
}

impl Error {
    #[track_caller]
    pub(crate) fn invalid_rust_type(rs_type: &str) -> Self {
        Self::new(format!("`{}` is not valid rust type", rs_type))
    }
    #[track_caller]
    pub(crate) fn col_type_not_found(col_name: &str) -> Self {
        Self::new(format!(
            "no type information found for column `{}`",
            col_name
        ))
    }
    #[track_caller]
    pub(crate) fn db_type_cannot_map(db_type: &str) -> Self {
        Self::new(format!("cannot map db type `{}` to rust type", db_type))
    }
    #[track_caller]
    pub(crate) fn parameter_col_not_found(query_name: &str) -> Self {
        Self::new(format!(
            "no parameter column found for query `{}`",
            query_name
        ))
    }
    #[track_caller]
    pub(crate) fn unsupported_annotation(annotation:&str)->Self{
        Self::new(format!(
            "query annotation `{}` is not supported",
            annotation
        ))
    }
    #[track_caller]
    pub(crate) fn any_error(txt: String) -> Self {
        const ISSUE_URL: &str = "https://github.com/tunamaguro/sqlc-rust-postgres/issues/new";
        let message = format!("It looks like you've encountered an unexpected bug. Please consider reporting this issue at {} so we can investigate further.\nDetail: {}",ISSUE_URL,txt);
        Self::new(message)
    }
    #[track_caller]
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: Location::caller(),
            backtrace: Backtrace::capture(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.message,
            self.location.file(),
            self.location.line()
        )?;

        if self.backtrace.status() == BacktraceStatus::Captured {
            write!(f, "\nBacktrace:\n{}", self.backtrace)?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}
