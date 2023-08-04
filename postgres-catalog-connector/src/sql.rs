mod fkey;
mod predicate_builder;
mod utils;

use indexmap::IndexMap;
use ndc_client::models::{self, Expression};
use sqlparser::ast::{
    Expr, ObjectName, Query, Select, SelectItem, SetExpr, Statement, TableAlias, TableFactor,
    TableWithJoins, Value,
};
use std::str::FromStr;

use crate::error::ServerError;
use crate::tables::SupportedCollection;

use fkey::get_fkey_query;
use predicate_builder::get_predicate_expression;
use utils::{get_sql_function_expression, get_sql_query, get_sql_quoted_identifier};

pub fn build_sql_query(request: &models::QueryRequest) -> Result<Statement, ServerError> {
    let collection = SupportedCollection::from_str(&request.collection);

    match collection {
        Ok(t) => Ok(Statement::Query(get_node_subquery(&request.query, &t)?)),
        Err(_) => Err(ServerError::BadRequest("unknown table".into())),
    }
}

pub fn get_node_subquery(
    query: &ndc_client::models::Query,
    collection: &SupportedCollection,
) -> Result<Box<Query>, ServerError> {
    // query wrapper projection
    let wrapper_projection = match &query.fields {
        Some(f) => {
            let rows_subquery = get_rows_json_subquery(f.clone(), query, collection);
            match rows_subquery {
                Ok(q) => vec![SelectItem::ExprWithAlias {
                    expr: Expr::Subquery(q),
                    alias: get_sql_quoted_identifier("rows"),
                }],
                Err(e) => return Err(e), // propogate the error sent by the subquery function
            }
        }
        None => return Err(ServerError::BadRequest("fields must be present".into())),
    };
    // build the wrapper query
    let wrapper_subquery = get_sql_query(wrapper_projection, vec![], None, None, None, None);

    // projection for the node subquery
    let node_projection = vec![SelectItem::ExprWithAlias {
        expr: get_sql_function_expression(
            "COALESCE",
            vec![
                get_sql_function_expression(
                    "json_agg",
                    vec![get_sql_function_expression(
                        "to_json",
                        vec![Expr::Identifier(get_sql_quoted_identifier("_wrapper"))],
                        None,
                    )],
                    None,
                ),
                get_sql_function_expression("json_build_array", vec![], None),
            ],
            None,
        ),
        alias: get_sql_quoted_identifier("_node"),
    }];
    // from clause of the node subquery
    let node_from = vec![TableWithJoins {
        relation: TableFactor::Derived {
            lateral: false,
            subquery: wrapper_subquery,
            alias: Some(TableAlias {
                columns: vec![],
                name: get_sql_quoted_identifier("_wrapper"),
            }),
        },
        joins: vec![],
    }];
    // get the node subquery
    let node_subquery = Box::new(Query {
        with: None,
        body: Box::new(SetExpr::Select(Box::new(Select {
            distinct: None,
            top: None,
            projection: node_projection,
            into: None,
            from: node_from,
            lateral_views: vec![],
            selection: None,
            group_by: vec![],
            cluster_by: vec![],
            distribute_by: vec![],
            sort_by: vec![],
            having: None,
            qualify: None,
            named_window: vec![],
        }))),
        limit: None,
        offset: None,
        order_by: vec![],
        locks: vec![],
        fetch: None,
    });

    Ok(node_subquery)
}

// get the subquery to to get rows json
fn get_rows_json_subquery(
    fields: IndexMap<String, models::Field>,
    query: &models::Query,
    table: &SupportedCollection,
) -> Result<Box<Query>, ServerError> {
    let row_subquery = match table {
        SupportedCollection::Columns | SupportedCollection::Tables => get_rows_query(query, table),
        SupportedCollection::ForeignKeys {} => get_fkey_query(query, table),
    };

    let rows_json_projection = vec![SelectItem::ExprWithAlias {
        expr: get_sql_function_expression(
            "COALESCE",
            vec![
                get_sql_function_expression(
                    "json_agg",
                    vec![if fields.is_empty() {
                        get_sql_function_expression("json_build_object", vec![], None)
                    } else {
                        get_sql_function_expression(
                            "to_json",
                            vec![Expr::Identifier(get_sql_quoted_identifier("_rows"))],
                            None,
                        )
                    }],
                    None,
                ),
                get_sql_function_expression("json_build_array", vec![], None),
            ],
            None,
        ),
        alias: get_sql_quoted_identifier("rows"),
    }];
    let rows_json_from = vec![TableWithJoins {
        joins: vec![],
        relation: TableFactor::Derived {
            lateral: false,
            subquery: match row_subquery {
                Ok(q) => q,
                Err(_) => todo!("todo"),
            },
            alias: Some(TableAlias {
                name: get_sql_quoted_identifier("_rows"),
                columns: vec![],
            }),
        },
    }];

    Ok(get_sql_query(
        rows_json_projection,
        rows_json_from,
        None,
        None,
        None,
        None,
    ))
}

pub fn get_rows_query(
    query: &ndc_client::models::Query,
    table: &SupportedCollection,
) -> Result<Box<Query>, ServerError> {
    /*Build Predicate*/
    // start with a custom predicate to ignore tables from information_schema and pg_catalog
    let mut predicate = Expression::And {
        expressions: vec![
            (Expression::BinaryComparisonOperator {
                column: Box::new(models::ComparisonTarget::RootCollectionColumn {
                    name: "table_schema".into(),
                }),
                operator: Box::new(models::BinaryComparisonOperator::Other {
                    name: "nlike".into(),
                }),
                value: Box::new(models::ComparisonValue::Scalar {
                    value: "pg_%".into(),
                }),
            }),
            (Expression::BinaryComparisonOperator {
                column: Box::new(models::ComparisonTarget::RootCollectionColumn {
                    name: "table_schema".into(),
                }),
                operator: Box::new(models::BinaryComparisonOperator::Other {
                    name: "nlike".into(),
                }),
                value: Box::new(models::ComparisonValue::Scalar {
                    value: "information_schema".into(),
                }),
            }),
        ],
    };
    // append the actual predicate coming from the query
    predicate = match &query.predicate {
        Some(p) => models::Expression::And {
            expressions: vec![predicate, p.clone()],
        },
        None => predicate,
    };
    // get the predicate expression required by the sqlx client
    let filter_predicate = get_predicate_expression(&predicate, "_origin");

    // from clause
    let rows_from = vec![TableWithJoins {
        joins: vec![],
        relation: TableFactor::Table {
            // note: assuming the table name is not aliased in any way, will need to change this
            name: ObjectName(vec![
                get_sql_quoted_identifier(table.get_schema_name()),
                get_sql_quoted_identifier(table.get_table_name()),
            ]),
            alias: Some(TableAlias {
                name: get_sql_quoted_identifier("_origin"),
                columns: vec![],
            }),
            args: None,
            with_hints: vec![],
        },
    }];

    // fields
    let binding = IndexMap::new();
    let fields = match &query.fields {
        Some(f) => f,
        None => &binding,
    };
    let rows_projection = if fields.is_empty() {
        vec![SelectItem::UnnamedExpr(Expr::Value(Value::Null))]
    } else {
        fields
            .iter()
            .map(|(alias, field)| SelectItem::ExprWithAlias {
                expr: get_sql_function_expression(
                    "json_build_object",
                    vec![
                        Expr::Value(Value::SingleQuotedString("value".to_string())),
                        match field {
                            models::Field::Column { column, .. } => {
                                let table_info = table.get_table_info();
                                let column_info = table_info
                                    .columns
                                    .iter()
                                    .find(|c| c.name == column.clone())
                                    .expect("column should be in table");
                                Expr::CompoundIdentifier(vec![
                                    get_sql_quoted_identifier("_origin"),
                                    get_sql_quoted_identifier(&column_info.name),
                                ])
                            }
                            models::Field::Relationship { .. } => todo!(),
                        },
                    ],
                    None,
                ),
                alias: get_sql_quoted_identifier(alias),
            })
            .collect()
    };

    Ok(get_sql_query(
        rows_projection,
        rows_from,
        Some(filter_predicate),
        None,
        query.limit,
        query.offset,
    ))
}
