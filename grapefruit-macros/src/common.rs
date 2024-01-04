use std::collections::HashMap;

use crate::util::string_to_bool;


pub(crate) enum TableAttribute {
    Table,
    TableId,
    TableColumn,
}

impl TableAttribute {
    pub fn name(&self) -> &str {
        match self {
            TableAttribute::Table => "table",
            TableAttribute::TableId => "id",
            TableAttribute::TableColumn => "column",
        }
    }

    pub fn has_attr(&self, attr: &str) -> bool {
        self.attrs().contains_key(attr)
    }

    pub fn check_value(&self, attr: &str, value: &str) -> (bool, String) {
        match self.attrs().get(attr) {
            Some(v) => match v {
                Some(vv) => {
                    let msg = vv
                        .iter()
                        .map(|vvv| format!("`{}`", vvv))
                        .collect::<Vec<_>>()
                        .join(", ");
                    (vv.contains(&value), msg)
                }
                None => (true, "".into()),
            },
            None => (false, "".into()),
        }
    }

    pub fn attrs(&self) -> HashMap<&str, Option<Vec<&str>>> {
        let bool_value = Some(vec!["true", "false"]);
        let mut map = HashMap::new();
        match self {
            TableAttribute::Table => {
                map.insert("name", None);
                map.insert("curd", bool_value);
            }
            TableAttribute::TableId => {
                map.insert("name", None);
                map.insert(
                    "id_type",
                    Some(vec!["auto", "generator"]),
                );
            }
            TableAttribute::TableColumn => {
                let strategy = Some(vec!["default", "not_null", "never"]);
                map.insert("name", None);
                map.insert("select", bool_value.clone());
                map.insert("ignore", bool_value.clone());
                map.insert("insert_strateg", strategy.clone());
                map.insert("update_strateg", strategy);
                map.insert(
                    "fill",
                    Some(vec!["default", "insert", "update", "insert_and_update"]),
                );
                map.insert("is_logic_delete", bool_value.clone());
                map.insert("version", bool_value);
            }
        };
        map
    }
}

#[derive(Debug)]
pub struct ColumnInformation {
    pub field: syn::Field,
    pub field_type: String,
    pub name: String,
    pub alias: Option<String>,
    pub attribute: ColumnAttribute,
    pub table_id: Option<TableId>,
    pub table_column: Option<TableColumn>,
}

impl ColumnInformation {
    pub fn new(
        field: syn::Field,
        field_type: String,
        name: String,
        attribute: &ColumnAttribute,
    ) -> Self {
        let alias = attribute.name().or(Some(name.clone()));
        Self {
            field,
            field_type,
            name,
            alias,
            attribute: attribute.clone(),
            table_id: attribute.get_table_id(),
            table_column: attribute.get_table_column(),
        }
    }

    pub fn is_table_id(&self) -> bool {
        matches!(self.attribute, ColumnAttribute::TableId(_))
    }

    pub fn get_table_id(&self) ->TableId {
        self.attribute.get_table_id().unwrap()
    }

    pub fn get_table_column(&self) -> TableColumn {
        self.attribute.get_table_column().unwrap()
    }

    pub fn is_table_column(&self) -> bool {
        matches!(self.attribute, ColumnAttribute::TableColumn(_))
    }
    // pub fn is_table(&self) -> bool {
    //     matches!(self.attribute, ColumnAttribute::Table(_))
    // }

    // pub fn alias(&self) -> String {
    //     self.alias.clone().unwrap_or_else(|| self.name.clone())
    // }

    pub fn is_logic_delete(&self) -> bool {
        self.attribute.is_logic_delete()
    }

    pub fn is_version(&self) -> bool {
        self.attribute.is_version()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnAttribute {
    Table(Table),
    TableId(TableId),
    TableColumn(TableColumn),
}

impl ColumnAttribute {
    pub fn name(&self) -> Option<String> {
        match self {
            ColumnAttribute::TableId(table_id) => table_id.name.clone(),
            ColumnAttribute::TableColumn(table_field) => table_field.name.clone(),
            ColumnAttribute::Table(s) => s.name.clone(),
        }
    }

    pub(crate) fn from_attribute(
        attribute: &TableAttribute,
        map: HashMap<String, String>,
    ) -> ColumnAttribute {
        match attribute {
            TableAttribute::Table => ColumnAttribute::Table(Table::from_map(map)),
            TableAttribute::TableId => ColumnAttribute::TableId(TableId::from_map(map)),
            TableAttribute::TableColumn => ColumnAttribute::TableColumn(TableColumn::from_map(map)),
        }
    }

    pub fn get_table(&self) -> Option<Table> {
        match self {
            ColumnAttribute::Table(table) => Some(table.clone()),
            _ => None,
        }
    }

    pub fn get_table_id(&self) -> Option<TableId> {
        match self {
            ColumnAttribute::TableId(table_id) => Some(table_id.clone()),
            _ => None,
        }
    }

    pub fn get_table_column(&self) -> Option<TableColumn> {
        match self {
            ColumnAttribute::TableColumn(table_column) => Some(table_column.clone()),
            _ => None,
        }
    }

    pub fn is_logic_delete(&self) -> bool{
        match self {
            ColumnAttribute::TableColumn(table_column) => table_column.is_logic_delete,
            _ => false,
        }
    }

    pub fn is_version(&self) -> bool {
        match self {
            ColumnAttribute::TableColumn(table_column) => table_column.version,
            _ => false,
        }
    }
}

impl Default for ColumnAttribute {
    fn default() -> Self {
        ColumnAttribute::TableColumn(TableColumn::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: Option<String>,
    pub curd: bool,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            name: None,
            curd: false,
        }
    }
}

impl Table {
    pub(crate) fn from_map(map: HashMap<String, String>) -> Table {
        let mut table_id = Table::default();
        map.get("name")
            .map(|name| table_id.name = Some(name.clone()));
        map.get("curd")
            .map(|curd| table_id.curd = string_to_bool(curd.as_str()).unwrap_or(false));
        table_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableId {
    pub name: Option<String>,
    pub id_type: String,
}
impl TableId {
    pub(crate) fn from_map(map: HashMap<String, String>) -> TableId {
        let mut table_id = TableId::default();
        map.get("name")
            .map(|name| table_id.name = Some(name.clone()));
        map.get("id_type")
            .map(|id_type| table_id.id_type = id_type.clone());
        table_id
    }
}

impl Default for TableId {
    fn default() -> Self {
        Self {
            name: None,
            id_type: "auto".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableColumn {
    pub name: Option<String>,
    pub select: bool,
    pub ignore: bool,
    pub insert_strateg: String,
    pub update_strateg: String,
    pub fill: String,
    pub is_logic_delete: bool,
    pub version: bool,
}
impl TableColumn {
    pub(crate) fn from_map(attr_value_map: HashMap<String, String>) -> TableColumn {
        let mut table_field = TableColumn::default();
        attr_value_map
            .get("name")
            .map(|name| table_field.name = Some(name.clone()));
        attr_value_map.get("select").map(|select| {
            table_field.select = string_to_bool(select.as_str()).unwrap_or(true);
        });
        attr_value_map.get("ignore").map(|ignore| {
            table_field.ignore = string_to_bool(ignore.as_str()).unwrap_or(false);
        });
        attr_value_map
            .get("insert_strateg")
            .map(|insert_strateg| table_field.insert_strateg = insert_strateg.clone());
        attr_value_map
            .get("update_strateg")
            .map(|update_strateg| table_field.update_strateg = update_strateg.clone());
        attr_value_map
            .get("fill")
            .map(|fill| table_field.fill = fill.clone());
        attr_value_map
            .get("is_logic_delete")
            .map(|is_logic_delete| {
                table_field.is_logic_delete =
                    string_to_bool(is_logic_delete.as_str()).unwrap_or(false)
            });
        attr_value_map
            .get("version")
            .map(|version| table_field.version = string_to_bool(version.as_str()).unwrap_or(false));
        table_field
    }
}

impl Default for TableColumn {
    fn default() -> Self {
        Self {
            name: None,
            select: true,
            ignore: false,
            insert_strateg: "default".into(),
            update_strateg: "default".into(),
            fill: "default".into(),
            is_logic_delete: false,
            version: false,
        }
    }
}
