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
pub enum SupportedTable {
    Tables,
    Columns,
}
// the underlying table names of these tables in information_schema
pub const TABLES: &str = "tables";
pub const COLUMNS: &str = "columns";
impl SupportedTable {
    // gets the name of the underlying table from enum
    pub fn get_table_name(&self) -> String {
        match self {
            SupportedTable::Tables => TABLES.to_string(),
            SupportedTable::Columns => COLUMNS.to_string(),
        }
    }

    // gets the schema of the underlying table from enum
    pub fn get_schema_name(&self) -> String {
        "information_schema".to_string()
    }

    // gets the columns of the underlying table from enum
    pub fn get_columns(&self) -> Vec<ColumnInfo> {
        match self {
            SupportedTable::Tables => {
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
            SupportedTable::Columns => {
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

impl ToString for SupportedTable {
    fn to_string(&self) -> String {
        match self {
            SupportedTable::Tables => TABLES.into(),
            SupportedTable::Columns => COLUMNS.into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseSupportedTableErr;
impl FromStr for SupportedTable {
    type Err = ParseSupportedTableErr;

    fn from_str(s: &str) -> Result<SupportedTable, ParseSupportedTableErr> {
        match s {
            TABLES => Ok(SupportedTable::Tables),
            COLUMNS => Ok(SupportedTable::Columns),
            _ => Err(ParseSupportedTableErr),
        }
    }
}
