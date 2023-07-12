use ndc_client::models;
use std::{
    collections::{HashMap},
};

use axum::{
    Json
};

pub const ROUTENAME: &str = "/schema";

pub async fn handler() -> Json<models::SchemaResponse> {

    println!("received schema request");

    // scalar types
    let scalar_types = HashMap::from_iter([
        (
            "String".into(),
            models::ScalarType {
                aggregate_functions: HashMap::new(),
                comparison_operators: HashMap::from_iter([(
                    "like".into(),
                    models::ComparisonOperatorDefinition {
                        argument_type: models::Type::Named {
                            name: "String".into(),
                        },
                    },
                )]),
                update_operators: HashMap::new(),
            },
        ),
        (
            "Int".into(),
            models::ScalarType {
                aggregate_functions: HashMap::from_iter([
                    (
                        "max".into(),
                        models::AggregateFunctionDefinition {
                            result_type: models::Type::Nullable {
                                underlying_type: Box::new(models::Type::Named {
                                    name: "Int".into(),
                                }),
                            },
                        },
                    ),
                    (
                        "min".into(),
                        models::AggregateFunctionDefinition {
                            result_type: models::Type::Nullable {
                                underlying_type: Box::new(models::Type::Named {
                                    name: "Int".into(),
                                }),
                            },
                        },
                    ),
                ]),
                comparison_operators: HashMap::from_iter([]),
                update_operators: HashMap::new(),
            },
        ),
    ]);

        let column_type = models::ObjectType {
        description: Some("Postgres column definition".into()),
        fields: HashMap::from_iter([
            (
                "table_name".into(),
                models::ObjectField {
                    description: Some("Name of the Postgres table".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named { name: "String".into() },
                },
            ),
            (
                "table_schema".into(),
                models::ObjectField {
                    description: Some("Name of the schema of the Postgres table".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "column_name".into(),
                models::ObjectField {
                    description: Some("Name of the table column".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named { name: "String".into() },
                },
            ),
            (
                "comment".into(),
                models::ObjectField {
                    description: Some("Comment of the table column".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named { name: "String".into() },
                },
            ),
            (
                "table".into(),
                models::ObjectField {
                    description: Some("Comment of the table column".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named { name: "table".into() },
                },

            ),
        ]),
    };

    let table_type = models::ObjectType {
        description: Some("Postgres table definition".into()),
        fields: HashMap::from_iter([
            (
                "table_name".into(),
                models::ObjectField {
                    description: Some("Name of the Postgres table".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named { name: "String".into() },
                },
            ),
            (
                "table_schema".into(),
                models::ObjectField {
                    description: Some("Name of the schema of the Postgres table".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "comment".into(),
                models::ObjectField {
                    description: Some("Name of the Postgres table".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Nullable { underlying_type: Box::new(models::Type::Named { name: "String".into(), }) } ,
                },
            ),
            (
                "columns".into(),
                models::ObjectField {
                    description: Some("The article's author ID".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Array { element_type: Box::new(models::Type::Named { name: "column".into(), }) } ,
                },
            ),
        ]),
    };
    // ANCHOR_END: schema_object_type_author
    // ANCHOR: schema_object_types
    let object_types = HashMap::from_iter([
        ("table".into(), table_type),
        ("column".into(), column_type),
    ]);


    let tables_table = models::TableInfo {
        name: "tables".into(),
        description: Some("A collection of Postgres tables".into()),
        table_type: "table".into(),
        arguments: HashMap::new(),
        deletable: false,
        insertable_columns: None,
        updatable_columns: None,
        foreign_keys: HashMap::new(),
        uniqueness_constraints: HashMap::from_iter([(
            "TableSchemaName".into(),
            models::UniquenessConstraint {
                unique_columns: vec!["table_schema".into(), "table_name".into()],
            },
        )]),
    };

    let columns_table = models::TableInfo {
        name: "columns".into(),
        description: Some("A collection of Postgres columns".into()),
        table_type: "column".into(),
        arguments: HashMap::new(),
        deletable: false,
        insertable_columns: None,
        updatable_columns: None,
        foreign_keys: HashMap::from_iter([(
            "ColumnToTable".into(),
            models::ForeignKeyConstraint {
                column_mapping: HashMap::from_iter([("table_schema".into(), "table_schema".into()), ("table_name".into(), "table_name".into())]),
                foreign_table: "table".into(),
            }
        )]),
        uniqueness_constraints: HashMap::from_iter([(
            "ColumnName".into(),
            models::UniquenessConstraint {
                unique_columns: vec!["table_schema".into(), "table_name".into(), "column_name".into()],
            },
        )]),
    };

    let tables = vec![tables_table, columns_table];

    // ANCHOR: schema_commands
    let commands = vec![];
    // ANCHOR_END: schema_commands

    Json(models::SchemaResponse {
        scalar_types,
        object_types,
        tables,
        commands,
    })
}