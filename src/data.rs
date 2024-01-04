use std::fmt;

use crate::GrapefruitResult;

#[derive(Debug)]
pub struct ExecResult {
    pub(crate) result: ExecResultHolder,
}

#[cfg(feature = "postgres")]
impl From<sqlx::postgres::PgQueryResult> for ExecResult {
    fn from(value: sqlx::postgres::PgQueryResult) -> Self {
        Self {
            result: ExecResultHolder::Postgres(value),
        }
    }
}

#[cfg(feature = "mysql")]
impl From<sqlx::mysql::MySqlQueryResult> for ExecResult {
    fn from(value: sqlx::mysql::MySqlQueryResult) -> Self {
        Self {
            result: ExecResultHolder::MySql(value),
        }
    }
}

#[cfg(feature = "sqlite")]
impl From<sqlx::sqlite::SqliteQueryResult> for ExecResult {
    fn from(value: sqlx::sqlite::SqliteQueryResult) -> Self {
        Self {
            result: ExecResultHolder::Sqlite(value),
        }
    }
}

impl ExecResult {
    pub fn last_insert_id(&self) -> Option<u64> {
        match self.result {
            #[cfg(feature = "mysql")]
            ExecResultHolder::MySql(ref result) => Some(result.last_insert_id()),
            #[cfg(feature = "postgres")]
            ExecResultHolder::Postgres(_) => None,
            #[cfg(feature = "sqlite")]
            ExecResultHolder::Sqlite(_) => None,
        }
    }

    pub fn rows_affected(&self) -> u64 {
        match self.result {
            #[cfg(feature = "mysql")]
            ExecResultHolder::MySql(ref result) => result.rows_affected(),
            #[cfg(feature = "postgres")]
            ExecResultHolder::Postgres(ref result) => result.rows_affected(),
            #[cfg(feature = "sqlite")]
            ExecResultHolder::Sqlite(ref result) => result.rows_affected(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.rows_affected() > 0
    }
}

#[derive(Debug)]
pub(crate) enum ExecResultHolder {
    #[cfg(feature = "mysql")]
    MySql(sqlx::mysql::MySqlQueryResult),
    #[cfg(feature = "postgres")]
    Postgres(sqlx::postgres::PgQueryResult),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::sqlite::SqliteQueryResult),
}

#[derive(Debug)]
pub struct QueryResults {
    pub results: Vec<QueryResult>,
}

impl QueryResults {
    pub fn try_get<T>(&self) -> GrapefruitResult<Vec<T>>
    where
        T: crate::TryGetable,
    {
        if self.results.is_empty() {
            return Ok(vec![]);
        }
        let mut values = Vec::with_capacity(self.results.len());
        for result in self.results.iter() {
            let t = result.try_get::<T>()?.expect("get val error!");
            values.push(t)
        }
        Ok(values)
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub row: Option<QueryRow>,
}

impl QueryResult {
    pub fn try_get<T>(&self) -> GrapefruitResult<Option<T>>
    where
        T: crate::TryGetable,
    {
        T::try_get(self)
    }
}

#[cfg(feature = "postgres")]
impl From<sqlx::postgres::PgRow> for QueryResult {
    fn from(value: sqlx::postgres::PgRow) -> Self {
        Self {
            row: Some(QueryRow::Postgres(value)),
        }
    }
}

#[cfg(feature = "mysql")]
impl From<sqlx::mysql::MySqlRow> for QueryResult {
    fn from(value: sqlx::mysql::MySqlRow) -> Self {
        Self {
            row: Some(QueryRow::MySql(value)),
        }
    }
}

#[cfg(feature = "sqlite")]
impl From<sqlx::sqlite::SqliteRow> for QueryResult {
    fn from(value: sqlx::sqlite::SqliteRow) -> Self {
        Self {
            row: Some(QueryRow::Sqlite(value)),
        }
    }
}

#[cfg(feature = "postgres")]
impl From<Option<sqlx::postgres::PgRow>> for QueryResult {
    fn from(value: Option<sqlx::postgres::PgRow>) -> Self {
        let row = match value {
            Some(v) => Some(QueryRow::Postgres(v)),
            None => None,
        };
        Self { row }
    }
}

#[cfg(feature = "mysql")]
impl From<Option<sqlx::mysql::MySqlRow>> for QueryResult {
    fn from(value: Option<sqlx::mysql::MySqlRow>) -> Self {
        let row = match value {
            Some(v) => Some(QueryRow::MySql(v)),
            None => None,
        };
        Self { row }
    }
}

#[cfg(feature = "sqlite")]
impl From<Option<sqlx::sqlite::SqliteRow>> for QueryResult {
    fn from(value: Option<sqlx::sqlite::SqliteRow>) -> Self {
        let row = match value {
            Some(v) => Some(QueryRow::Sqlite(v)),
            None => None,
        };
        Self { row }
    }
}

#[cfg(feature = "postgres")]
impl From<Vec<sqlx::postgres::PgRow>> for QueryResults {
    fn from(values: Vec<sqlx::postgres::PgRow>) -> Self {
        let results = values
            .into_iter()
            .map(|row| row.into())
            .collect::<Vec<QueryResult>>();
        QueryResults { results }
    }
}

#[cfg(feature = "mysql")]
impl From<Vec<sqlx::mysql::MySqlRow>> for QueryResults {
    fn from(values: Vec<sqlx::mysql::MySqlRow>) -> Self {
        let results = values
            .into_iter()
            .map(|row| row.into())
            .collect::<Vec<QueryResult>>();
        QueryResults { results }
    }
}

#[cfg(feature = "sqlite")]
impl From<Vec<sqlx::sqlite::SqliteRow>> for QueryResults {
    fn from(values: Vec<sqlx::sqlite::SqliteRow>) -> Self {
        let results = values
            .into_iter()
            .map(|row| row.into())
            .collect::<Vec<QueryResult>>();
        QueryResults { results }
    }
}

pub enum QueryRow {
    #[cfg(feature = "mysql")]
    MySql(sqlx::mysql::MySqlRow),
    #[cfg(feature = "postgres")]
    Postgres(sqlx::postgres::PgRow),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::sqlite::SqliteRow),
}

#[allow(unused_variables)]
impl fmt::Debug for QueryRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "mysql")]
            Self::MySql(row) => write!(f, "{row:?}"),
            #[cfg(feature = "postgres")]
            Self::Postgres(_) => write!(f, "QueryRow::SqlxPostgres cannot be inspected"),
            #[cfg(feature = "sqlite")]
            Self::Sqlite(_) => write!(f, "QueryRow::SqlxSqlite cannot be inspected"),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}
