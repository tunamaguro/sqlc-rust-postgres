/// sqlc annotation
/// See https://docs.sqlc.dev/en/stable/reference/query-annotations.html
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum QueryAnnotation {
    Exec,
    ExecResult,
    ExecRows,
    ExecLastId,
    Many,
    One,
    BatchExec,
    BatchMany,
    BatchOne,
    CopyFrom,
    Unknown(String),
}

impl std::fmt::Display for QueryAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            QueryAnnotation::Exec => ":exec",
            QueryAnnotation::ExecResult => ":execresult",
            QueryAnnotation::ExecRows => ":execrows",
            QueryAnnotation::ExecLastId => ":execlastid",
            QueryAnnotation::Many => ":many",
            QueryAnnotation::One => ":one",
            QueryAnnotation::BatchExec => ":batch",
            QueryAnnotation::BatchMany => ":batchmany",
            QueryAnnotation::BatchOne => ":batchone",
            QueryAnnotation::CopyFrom => ":copyfrom",
            QueryAnnotation::Unknown(s) => s,
        };
        f.write_str(txt)
    }
}

impl std::str::FromStr for QueryAnnotation {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let annotation = match s {
            ":exec" => QueryAnnotation::Exec,
            ":execresult" => QueryAnnotation::ExecResult,
            ":execrows" => QueryAnnotation::ExecRows,
            ":execlastid" => QueryAnnotation::ExecLastId,
            ":many" => QueryAnnotation::Many,
            ":one" => QueryAnnotation::One,
            ":batch" => QueryAnnotation::BatchExec,
            ":batchmany" => QueryAnnotation::BatchMany,
            ":batchone" => QueryAnnotation::BatchOne,
            ":copyfrom" => QueryAnnotation::CopyFrom,
            _ => QueryAnnotation::Unknown(s.to_string()),
        };
        Ok(annotation)
    }
}
