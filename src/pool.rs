use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    exec, snowflake::SnowflakeGenerator, ExecResult, IdentifierGenerator, MetaObjectHandler,
    Params, QueryResult, QueryResults, Value,
};

use url::Url;

use crate::{GrapefruitError, GrapefruitResult};

#[derive(Debug, Clone)]
pub enum Platform {
    #[cfg(feature = "mysql")]
    Mysql,
    #[cfg(feature = "postgres")]
    Postgres,
    #[cfg(feature = "sqlite")]
    Sqlite,
    Unsupported(String),
}

impl Default for Platform {
    fn default() -> Self {
        Platform::Unsupported("Unsupported platform".into())
    }
}

impl Platform {
    pub fn mark(&self, index: usize) -> String {
        match self {
            #[cfg(feature = "mysql")]
            Platform::Mysql => format!("?"),
            #[cfg(feature = "postgres")]
            Platform::Postgres => format!("${}", index),
            #[cfg(feature = "sqlite")]
            Platform::Sqlite => format!("?{}", index),
            _ => "".to_owned(),
        }
    }

    pub fn symbol(&self, column: &str) -> String {
        match self {
            #[cfg(feature = "mysql")]
            Platform::Mysql => format!("`{}`", column),
            #[cfg(feature = "postgres")]
            Platform::Postgres => format!("\"{}\"", column),
            #[cfg(feature = "sqlite")]
            Platform::Sqlite => format!("'{}'", column),
            _ => "".to_owned(),
        }
    }
}

impl<'a> TryFrom<&'a str> for Platform {
    type Error = GrapefruitError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let url = Url::parse(value);
        match url {
            Ok(url) => {
                let scheme = url.scheme();
                match scheme {
                    #[cfg(feature = "mysql")]
                    "mysql" => Ok(Platform::Mysql),
                    #[cfg(feature = "postgres")]
                    "postgres" => Ok(Platform::Postgres),
                    #[cfg(feature = "sqlite")]
                    "sqlite" => Ok(Platform::Sqlite),
                    _ => Ok(Platform::Unsupported(scheme.to_string())),
                }
            }
            Err(e) => Err(GrapefruitError::UrlParseError(e)),
        }
    }
}

#[derive(Clone)]
pub struct MetaObject {
    insert_fill: HashMap<String, Value>,
    update_fill: HashMap<String, Value>,
}

impl Default for MetaObject {
    fn default() -> Self {
        Self {
            insert_fill: HashMap::new(),
            update_fill: HashMap::new(),
        }
    }
}

impl MetaObject {
    pub fn try_get_insert_fill(&self, key: &str) -> GrapefruitResult<Value> {
        let Some(v) = self.insert_fill.get(key) else {
            return Err(GrapefruitError::MetaObjectNotFound(key.to_string()));
        };
        Ok(v.clone())
    }
    pub fn try_get_update_fill(&self, key: &str) -> GrapefruitResult<Value> {
        let Some(v) = self.update_fill.get(key) else {
            return Err(GrapefruitError::MetaObjectNotFound(key.to_string()));
        };
        Ok(v.clone())
    }
    pub fn set_insert_fill(&mut self, key: &str, value: Value) {
        self.insert_fill.insert(key.to_owned(), value);
    }

    pub fn set_update_fill(&mut self, key: &str, value: Value) {
        self.update_fill.insert(key.to_owned(), value);
    }
}

#[derive(Clone)]
pub struct GrapefruitOptions {
    pub(crate) url: String,
    pub(crate) max_connections: u32,
    pub(crate) acquire_timeout: Duration,
    pub(crate) min_connections: u32,
    pub(crate) max_lifetime: Option<Duration>,
    pub(crate) idle_timeout: Option<Duration>,
    pub(crate) platform: Platform,
    pub(crate) identifier_generator: Arc<Box<dyn IdentifierGenerator>>,
    pub(crate) meta_object_handler: Option<Arc<Box<dyn MetaObjectHandler>>>,
    pub(crate) meta_object: MetaObject,
}

impl GrapefruitOptions {
    pub fn new(url: &str) -> Self {
        let platform = Platform::try_from(url).expect("Failed to parse platform from url");
        Self {
            url: url.to_owned(),
            max_connections: 10,
            acquire_timeout: Duration::from_secs(5),
            min_connections: 5,
            max_lifetime: None,
            idle_timeout: None,
            platform,
            identifier_generator: Arc::new(Box::new(SnowflakeGenerator::default())),
            meta_object_handler: None,
            meta_object: MetaObject::default(),
        }
    }

    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    pub fn with_acquire_timeout(mut self, acquire_timeout: Duration) -> Self {
        self.acquire_timeout = acquire_timeout;
        self
    }

    pub fn with_min_connections(mut self, min_connections: u32) -> Self {
        self.min_connections = min_connections;
        self
    }

    pub fn with_max_lifetime(mut self, max_lifetime: Duration) -> Self {
        self.max_lifetime = Some(max_lifetime);
        self
    }

    pub fn with_idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.idle_timeout = Some(idle_timeout);
        self
    }

    pub fn with_identifier_generator(mut self, generator: Box<dyn IdentifierGenerator>) -> Self {
        self.identifier_generator = Arc::new(generator);
        self
    }

    pub fn with_meta_object_handler(mut self, handler: Box<dyn MetaObjectHandler>) -> Self {
        self.meta_object_handler = Some(Arc::new(handler));
        if let Some(handler) = &self.meta_object_handler {
            handler.insert_fill(&mut self.meta_object);
            handler.update_fill(&mut self.meta_object);
        }
        self
    }
}

#[derive(Debug)]
pub enum PlatformPool {
    #[cfg(feature = "mysql")]
    Mysql(sqlx::MySqlPool),
    #[cfg(feature = "postgres")]
    Postgres(sqlx::PgPool),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::SqlitePool),
}

impl PlatformPool {
    pub async fn new(options: &GrapefruitOptions) -> GrapefruitResult<PlatformPool> {
        let platform = &options.platform;
        match platform {
            #[cfg(feature = "mysql")]
            Platform::Mysql => {
                let pool = sqlx::mysql::MySqlPoolOptions::new()
                    .max_connections(options.max_connections)
                    .min_connections(options.min_connections)
                    .acquire_timeout(options.acquire_timeout)
                    .max_lifetime(options.max_lifetime)
                    .idle_timeout(options.idle_timeout)
                    .connect(&options.url)
                    .await
                    .expect("Connect to postgres failed.");
                Ok(PlatformPool::Mysql(pool))
            }
            #[cfg(feature = "postgres")]
            Platform::Postgres => {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .max_connections(options.max_connections)
                    .min_connections(options.min_connections)
                    .acquire_timeout(options.acquire_timeout)
                    .max_lifetime(options.max_lifetime)
                    .idle_timeout(options.idle_timeout)
                    .connect(&options.url)
                    .await
                    .expect("Connect to postgres failed.");
                Ok(PlatformPool::Postgres(pool))
            }
            #[cfg(feature = "sqlite")]
            Platform::Sqlite => {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(options.max_connections)
                    .min_connections(options.min_connections)
                    .acquire_timeout(options.acquire_timeout)
                    .max_lifetime(options.max_lifetime)
                    .idle_timeout(options.idle_timeout)
                    .connect(&options.url)
                    .await
                    .expect("Connect to postgres failed.");
                Ok(PlatformPool::Sqlite(pool))
            }
            Platform::Unsupported(_) => Err(GrapefruitError::PlatformError(
                "unsupported platform".into(),
            )),
        }
    }

    pub async fn execute(&self, sql: &str, params: Params) -> GrapefruitResult<ExecResult> {
        let row: ExecResult = exec!(self, sql, params, execute);
        Ok(row)
    }

    pub async fn fetch_one(&self, sql: &str, params: Params) -> GrapefruitResult<QueryResult> {
        let row: QueryResult = exec!(self, sql, params, fetch_optional);
        Ok(row)
    }

    pub async fn fetch_all(&self, sql: &str, params: Params) -> GrapefruitResult<QueryResults> {
        let rows: QueryResults = exec!(self, sql, params, fetch_all);
        Ok(rows)
    }
}

#[macro_export]
macro_rules! exec {
    ($se:expr, $sql:expr, $params:expr, $fun:ident) => {{
        let res = match $se {
            #[cfg(feature = "mysql")]
            PlatformPool::Mysql(pool) => {
                let args =
                    <Params as sqlx::IntoArguments<'_, sqlx::MySql>>::into_arguments($params);
                let result = sqlx::query_with($sql, args).$fun(pool).await?;
                result.into()
            }
            #[cfg(feature = "postgres")]
            PlatformPool::Postgres(pool) => {
                let args =
                    <Params as sqlx::IntoArguments<'_, sqlx::Postgres>>::into_arguments($params);
                let result = sqlx::query_with($sql, args).$fun(pool).await?;
                result.into()
            }
            #[cfg(feature = "sqlite")]
            PlatformPool::Sqlite(pool) => {
                let args =
                    <Params as sqlx::IntoArguments<'_, sqlx::Sqlite>>::into_arguments($params);
                let result = sqlx::query_with($sql, args).$fun(pool).await?;
                result.into()
            }
        };
        res
    }};
}
