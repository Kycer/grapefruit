use std::collections::HashMap;

use sqlx::Row;

use crate::{GrapefruitError, GrapefruitResult, QueryResult, Value};

pub trait PrimaryKey: Send + Sync + Clone + 'static + Into<Value> {
    type Key
    where
        Self: Into<Value>;

    fn key(&self) -> Value;
}

macro_rules! impl_primary_key {
    ($ty: ty) => {
        impl PrimaryKey for $ty {
            type Key = $ty;

            fn key(&self) -> Value {
                self.to_owned().into()
            }
        }
    };
}

impl_primary_key!(i64);
impl_primary_key!(u64);
impl_primary_key!(String);

pub trait Entity: 'static + Send + Sync {
    fn table_info() -> TableInfo;

    fn table_name() -> String {
        Self::table_info().table_name.clone()
    }

    fn to_value(&self) -> HashMap<String, crate::Value>;

    fn columns() -> HashMap<String, ColumnInfo>;

    fn primary_key() -> Option<ColumnInfo>;

    fn logic_delete() -> Option<ColumnInfo>;

    fn version() -> Option<ColumnInfo>;

    fn insert_columns() -> Vec<String>;

    fn update_columns() -> Vec<String>;

    fn select_columns() -> Vec<String>;
}

pub trait Column {
    fn column_info(&self) -> Option<ColumnInfo>;

    fn alias(&self) -> GrapefruitResult<String> {
        let alias_op = self.column_info().map(|v| v.alias.to_string());
        let Some(alias) = alias_op else {
            return Err(GrapefruitError::PrimaryKeyNone(
                "column not set".to_string(),
            ));
        };
        Ok(alias)
    }

    fn alias_unwrap(&self) -> String {
        self.alias().unwrap()
    }

    fn column_type_unwrap(&self) -> ColumnType {
        self.column_info().unwrap().column_type
    }
}

pub trait TryGetable: Sized + Default {
    fn try_get(result: &QueryResult) -> GrapefruitResult<Option<Self>>;
}

macro_rules! impl_try_get {
    ( $type: ty, $default: expr ) => {
        impl crate::TryGetable for $type {
            fn try_get(result: &crate::QueryResult) -> crate::GrapefruitResult<Option<$type>> {
                let res = match result.row.as_ref() {
                    Some(v) => match v {
                        #[cfg(feature = "mysql")]
                        crate::QueryRow::MySql(row) => row.try_get(0)?,
                        #[cfg(feature = "postgres")]
                        crate::QueryRow::Postgres(row) => row.try_get(0)?,
                        #[cfg(feature = "sqlite")]
                        crate::QueryRow::Sqlite(row) => row.try_get(0)?,
                    },
                    None => $default as $type,
                };
                Ok(Some(res))
            }
        }
    };
}

impl_try_get!(i8, 0);
impl_try_get!(i16, 0);
impl_try_get!(i32, 0);
impl_try_get!(i64, 0);
// impl_try_get!(u8);
// impl_try_get!(u16, 0);
// impl_try_get!(u32);
// impl_try_get!(u64);
impl_try_get!(f32, 0);
impl_try_get!(f64, 0);
impl_try_get!(bool, false);
impl_try_get!(String, "".to_string());
// impl_try_get!(&str, "");

#[derive(Debug, Clone, PartialEq)]
pub struct TableInfo {
    pub table_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnInfo {
    pub name: String,
    pub alias: String,
    pub ignore: bool,
    pub insert_strateg: ColumnStrategy,
    pub update_strateg: ColumnStrategy,
    pub column_type: ColumnType,
    pub fill: Fill,
    pub is_logic_delete: bool,
    pub version: bool,
}

impl ColumnInfo {
    pub fn is_ignore(&self) -> bool {
        self.ignore
    }

    pub fn is_logic_delete(&self) -> bool {
        self.is_logic_delete
    }

    pub fn is_version(&self) -> bool {
        self.version
    }

    pub fn is_table_id(&self) -> bool {
        matches!(self.column_type, ColumnType::TableId(_))
    }

    pub fn id_type(&self) -> IdType {
        self.column_type.get_id_type()
    }

}

impl Column for ColumnInfo {
    fn column_info(&self) -> Option<ColumnInfo> {
        Some(self.clone())
    }
}

impl Column for Option<ColumnInfo> {
    fn column_info(&self) -> Option<ColumnInfo> {
        self.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdType {
    Auto,
    Generator,
    Input,
}

impl IdType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "auto" => IdType::Auto,
            "generator" => IdType::Generator,
            "input" => IdType::Input,
            _ => IdType::Auto,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnStrategy {
    Default,
    NotNull,
    Never,
}

impl ColumnStrategy {
    pub fn from_str(s: &str) -> Self {
        match s {
            "default" => ColumnStrategy::Default,
            "not_null" => ColumnStrategy::NotNull,
            "never" => ColumnStrategy::Never,
            _ => ColumnStrategy::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fill {
    Default,
    Insert,
    Update,
    InsertAndUpdate,
}

impl Fill {
    pub fn from_str(s: &str) -> Self {
        match s {
            "default" => Fill::Default,
            "insert" => Fill::Insert,
            "update" => Fill::Update,
            "insert_and_update" => Fill::InsertAndUpdate,
            _ => Fill::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    TableId(IdType),
    TableColumn(String), // column type
}

impl ColumnType {
    pub fn logic_delete_value(&self) -> (Value, Value) {
        match self {
            ColumnType::TableId(_) => panic!("TableId does not have a logic delete value"),
            ColumnType::TableColumn(column_type) => {
                if column_type.contains("bool") {
                    (false.into(), true.into())
                } else if column_type.contains("i8") {
                    ((0 as i8).into(), (1 as i8).into())
                } else if column_type.contains("String") {
                    ("'N'".into(), "'Y'".into())
                } else {
                    panic!(
                        "Unsupported column type for logic delete value: {}",
                        column_type
                    );
                }
            }
        }
    }

    pub fn get_id_type(&self) -> IdType {
        match self {
            ColumnType::TableId(id_type) => id_type.clone(),
            ColumnType::TableColumn(_) => panic!("TableColumn does not have an id type"),
        }
    }
}
