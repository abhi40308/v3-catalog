use std::str::FromStr;

// An abstract table type for information about the supported tables
pub struct TableInfo {
    pub schema_name: String,
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
}

// An abstract column type for information about the supported tables
pub struct ColumnInfo {
    pub name: String,
    pub r#type: String,
}

// Tables supported by this data connector
pub enum SupportedCollection {
    Tables,
    Columns,
    ForeignKeys,
}
// the underlying table names of these tables in information_schema
pub const TABLES: &str = "tables";
pub const COLUMNS: &str = "columns";
pub const FOREIGN_KEYS: &str = "foreign_keys";
impl SupportedCollection {
    // gets the name of the underlying table from enum
    pub fn get_table_name(&self) -> String {
        match self {
            SupportedCollection::Tables => TABLES.to_string(),
            SupportedCollection::Columns => COLUMNS.to_string(),
            SupportedCollection::ForeignKeys => FOREIGN_KEYS.to_string(),
        }
    }

    // gets the schema of the underlying table from enum
    pub fn get_schema_name(&self) -> String {
        "information_schema".to_string()
    }

    // gets the columns of the underlying table from enum
    pub fn get_columns(&self) -> Vec<ColumnInfo> {
        match self {
            SupportedCollection::Tables => {
                vec![
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "table_name".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "table_schema".into(),
                    },
                ]
            }
            SupportedCollection::Columns => {
                vec![
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "table_name".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "table_schema".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "column_name".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "data_type".into(),
                    },
                ]
            }
            SupportedCollection::ForeignKeys => {
                vec![
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "schema_from".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "table_from".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "column_mapping".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "schema_to".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "table_to".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "fkey_name".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "on_update".into(),
                    },
                    ColumnInfo {
                        r#type: "String".into(),
                        name: "on_delete".into(),
                    },
                ]
            }
        }
    }

    pub fn get_table_info(&self) -> TableInfo {
        TableInfo {
            schema_name: self.get_schema_name(),
            table_name: self.get_table_name(),
            columns: self.get_columns(),
        }
    }
}

impl ToString for SupportedCollection {
    fn to_string(&self) -> String {
        match self {
            SupportedCollection::Tables => TABLES.into(),
            SupportedCollection::Columns => COLUMNS.into(),
            SupportedCollection::ForeignKeys => FOREIGN_KEYS.into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseSupportedCollectionErr;
impl FromStr for SupportedCollection {
    type Err = ParseSupportedCollectionErr;

    fn from_str(s: &str) -> Result<SupportedCollection, ParseSupportedCollectionErr> {
        match s {
            TABLES => Ok(SupportedCollection::Tables),
            COLUMNS => Ok(SupportedCollection::Columns),
            FOREIGN_KEYS => Ok(SupportedCollection::ForeignKeys),
            _ => Err(ParseSupportedCollectionErr),
        }
    }
}
