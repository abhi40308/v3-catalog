use ndc_client::models;
use std::collections::HashMap;

use axum::Json;

pub const ROUTENAME: &str = "/schema";

// TODO: get mod tables and get_schema to use the same tables
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
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
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
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "comment".into(),
                models::ObjectField {
                    description: Some("Comment of the table column".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "table".into(),
                models::ObjectField {
                    description: Some("Comment of the table column".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "table".into(),
                    },
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
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
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
                    r#type: models::Type::Nullable {
                        underlying_type: Box::new(models::Type::Named {
                            name: "String".into(),
                        }),
                    },
                },
            ),
            (
                "columns".into(),
                models::ObjectField {
                    description: Some("The article's author ID".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Array {
                        element_type: Box::new(models::Type::Named {
                            name: "column".into(),
                        }),
                    },
                },
            ),
        ]),
    };

    let foreign_key_type = models::ObjectType {
        description: Some("Postgres foreign keys definition".into()),
        fields: HashMap::from_iter([
            (
                "schema_from".into(),
                models::ObjectField {
                    description: Some(
                        "Name of the schema from which the foreign key exists".into(),
                    ),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "table_from".into(),
                models::ObjectField {
                    description: Some("Name of the table from which the foreign key exists".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "column_mapping".into(),
                models::ObjectField {
                    description: Some("Mapping of the columns with the foreign key".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        // TODO: This works as of now, but we should update this type to be something more suited,
                        // like jsonb
                        name: "String".into(),
                    },
                },
            ),
            (
                "schema_to".into(),
                models::ObjectField {
                    description: Some("Name of the schema to which the foreign key exists".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "table_to".into(),
                models::ObjectField {
                    description: Some("Name of the table to which the foreign key exists".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "fkey_name".into(),
                models::ObjectField {
                    description: Some("Name of the foreign key constraint".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "on_update".into(),
                models::ObjectField {
                    description: Some("On update clause".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
            (
                "on_delete".into(),
                models::ObjectField {
                    description: Some("On delete clause".into()),
                    arguments: HashMap::new(),
                    r#type: models::Type::Named {
                        name: "String".into(),
                    },
                },
            ),
        ]),
    };
    // ANCHOR_END: schema_object_type_author
    // ANCHOR: schema_object_types
    let object_types = HashMap::from_iter([
        ("tables".into(), table_type),
        ("columns".into(), column_type),
        ("foreign_keys".into(), foreign_key_type),
    ]);

    let database_url_argument: HashMap<String, models::ArgumentInfo> = HashMap::from_iter([(
        "database_url".into(),
        models::ArgumentInfo {
            description: Some(
                "The PG connection URI of the Postgres database that you wish to get entities from"
                    .into(),
            ),
            argument_type: models::Type::Named {
                name: "database_url".into(),
            },
        },
    )]);

    let tables_table = models::TableInfo {
        name: "tables".into(),
        description: Some("A collection of Postgres tables".into()),
        table_type: "table".into(),
        arguments: database_url_argument.clone(),
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
        arguments: database_url_argument.clone(),
        deletable: false,
        insertable_columns: None,
        updatable_columns: None,
        foreign_keys: HashMap::from_iter([(
            "ColumnToTable".into(),
            models::ForeignKeyConstraint {
                column_mapping: HashMap::from_iter([
                    ("table_schema".into(), "table_schema".into()),
                    ("table_name".into(), "table_name".into()),
                ]),
                foreign_table: "table".into(),
            },
        )]),
        uniqueness_constraints: HashMap::from_iter([(
            "ColumnName".into(),
            models::UniquenessConstraint {
                unique_columns: vec![
                    "table_schema".into(),
                    "table_name".into(),
                    "column_name".into(),
                ],
            },
        )]),
    };

    let foreign_keys_table = models::TableInfo {
        name: "foreign_keys".into(),
        description: Some("A collection of Postgres foreign keys".into()),
        table_type: "foreign_key".into(),
        arguments: database_url_argument.clone(),
        deletable: false,
        insertable_columns: None,
        updatable_columns: None,
        foreign_keys: HashMap::new(),
        uniqueness_constraints: HashMap::from_iter([(
            "ForeignKeyName".into(),
            models::UniquenessConstraint {
                unique_columns: vec!["fkey_name".into()],
            },
        )]),
    };

    let tables = vec![tables_table, columns_table, foreign_keys_table];

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
