use std::collections::HashMap;
use std::str::FromStr;
use sqlparser::ast::{Expr, Offset};
use ndc_client::models::{self, Expression};
use sqlparser::ast::{
    Query, Statement, SetExpr, Select, SelectItem, BinaryOperator, Ident, Value, UnaryOperator, TableAlias, TableWithJoins, TableFactor, ObjectName, Function, FunctionArg, FunctionArgExpr
};

use crate::error::ServerError;
use crate::tables::{SupportedTable};

pub fn build_sql_query(request: &models::QueryRequest) -> Result<Statement, ServerError> {

    let table = SupportedTable::from_str(&request.table);
    
    match table {
        Ok(t) => {
            Ok(Statement::Query(get_node_subquery(&request.query, &t)?))
        },
        Err(_) => Err(ServerError::BadRequest("unknown table".into()))
    }
    
}


pub fn get_node_subquery(query: &ndc_client::models::Query, table: &SupportedTable) -> Result<Box<Query>, ServerError> {

    // query wrapper projection
    let wrapper_projection = match &query.fields {
        Some(f) => {
            let rows_subquery = get_rows_json_subquery(f.clone(), query, table);
            match rows_subquery {
                Ok(q) => vec![SelectItem::ExprWithAlias {
                    expr: Expr::Subquery(q),
                    alias: get_sql_quoted_identifier("rows"),
                }],
                Err(e) => return Err(e) // propogate the error sent by the subquery function
            }
            
        },
        None => return Err(ServerError::BadRequest("fields must be present".into()))
    };
    // build the wrapper query
    let wrapper_subquery = get_sql_query(wrapper_projection, vec![], None, None, None);

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
                    )],
                ),
                get_sql_function_expression("json_build_array", vec![]),
            ],
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
    fields: HashMap<String, models::Field>,
    query: &models::Query,
    table: &SupportedTable,
) -> Result<Box<Query>, ServerError> {

    let row_subquery = get_rows_query(query, table);

    let rows_json_projection = vec![SelectItem::ExprWithAlias {
        expr: get_sql_function_expression(
            "COALESCE",
            vec![
                get_sql_function_expression(
                    "json_agg",
                    vec![if fields.is_empty() {
                        get_sql_function_expression("json_build_object", vec![])
                    } else {
                        get_sql_function_expression(
                            "to_json",
                            vec![Expr::Identifier(get_sql_quoted_identifier("_rows"))],
                        )
                    }],
                ),
                get_sql_function_expression("json_build_array", vec![]),
            ],
        ),
        alias: get_sql_quoted_identifier("rows"),
    }];
    let rows_json_from = vec![TableWithJoins {
        joins: vec![],
        relation: TableFactor::Derived {
            lateral: false,
            subquery: match row_subquery {
                Ok(q) => q,
                Err(_) => todo!("todo")
            },
            alias: Some(TableAlias {
                name: get_sql_quoted_identifier("_rows"),
                columns: vec![],
            }),
        },
    }];

    Ok(get_sql_query(rows_json_projection, rows_json_from, None, None, None))
}

pub fn get_rows_query(query: &ndc_client::models::Query, table: &SupportedTable) -> Result<Box<Query>, ServerError> {

    /*Build Predicate*/

    // start with a custom predicate to ignore tables from information_schema and pg_catalog
    let mut predicate =
     Expression::And { expressions: vec![(
        Expression::BinaryComparisonOperator{
        column: Box::new(models::ComparisonTarget::RootTableColumn { name: "table_schema".into() }),
        operator: Box::new(models::BinaryComparisonOperator::Other { name: "nlike".into() }),
        value: Box::new(models::ComparisonValue::Scalar{value: "pg_%".into()}),
      }), (
        Expression::BinaryComparisonOperator{
        column: Box::new(models::ComparisonTarget::RootTableColumn { name: "table_schema".into() }),
        operator: Box::new(models::BinaryComparisonOperator::Other { name: "nlike".into() }),
        value: Box::new(models::ComparisonValue::Scalar{value: "information_schema".into()}),
      })]
    };
    // append the actual predicate coming from the query
    predicate = match &query.predicate {
        Some(p) => models::Expression::And { expressions: vec![predicate, p.clone()] },
        None => predicate,
    };
    // get the predicate expression required by the sqlx client
    let filter_predicate =get_predicate_expression(&predicate);

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
    let binding = HashMap::new();
    let fields = match &query.fields {
        Some(f) => f,
        None => &binding
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
                    ),
                    alias: get_sql_quoted_identifier(alias),
                })
                .collect()
        };


    Ok(get_sql_query(rows_projection, rows_from, Some(filter_predicate), query.limit, query.offset))
}


// util function to build an SQL query from constructed parameters
fn get_sql_query(
    projection: Vec<SelectItem>,
    from: Vec<TableWithJoins>,
    predicate: Option<Expr>,
    limit: Option<u32>,
    offset: Option<u32>
) -> Box<Query> {
    Box::new(Query {
        with: None,
        body: Box::new(SetExpr::Select(Box::new(Select {
            selection: predicate,
            from,
            projection,
            lateral_views: vec![],
            distinct: None,
            top: None,
            into: None,
            group_by: vec![],
            cluster_by: vec![],
            distribute_by: vec![],
            sort_by: vec![],
            having: None,
            qualify: None,
            named_window: vec![],
        }))),
        limit: limit.map(|l| Expr::Value(Value::Number(l.to_string(), false))),
        offset: offset.map(|o| Offset{value: Expr::Value(Value::Number(o.to_string(), false)), rows: sqlparser::ast::OffsetRows::None}),
        fetch: None,
        locks: vec![],
        order_by: vec![], //todo
    })
}


// builds a predicate expression as expected by the sqlx client
fn get_predicate_expression(expr: &models::Expression) -> Expr {
    match expr {
        models::Expression::And { expressions } => expressions
            .iter()
            .map(|e| get_predicate_expression(e))
            .reduce(get_sql_and_expression)
            .map(|e| match e {
                Expr::BinaryOp {
                    op: BinaryOperator::And,
                    ..
                } => Expr::Nested(Box::new(e)),
                _ => e,
            })
            .unwrap_or_else(|| Expr::Value(Value::Boolean(true))),
        models::Expression::Or { expressions } => expressions
            .iter()
            .map(|e| get_predicate_expression(e))
            .reduce(get_sql_or_expr)
            .map(|e| match e {
                Expr::BinaryOp {
                    op: BinaryOperator::Or,
                    ..
                } => Expr::Nested(Box::new(e)),
                _ => e,
            })
            .unwrap_or_else(|| Expr::Value(Value::Boolean(false))),
        models::Expression::Not { expression } => Expr::UnaryOp {
            op: UnaryOperator::Not,
            expr: Box::new(get_predicate_expression(expression)),
        },
        models::Expression::UnaryComparisonOperator { .. } => todo!(),
        models::Expression::BinaryComparisonOperator {
            column,
            operator,
            value,
        } => {
            // this is silly. Todo: remove redundant boxes from input types.
            let operator = &**operator;
            let column = &**column;
            let value = &**value;


            let left = match column {
                models::ComparisonTarget::RootTableColumn { name } => {
                    Expr::CompoundIdentifier(vec![
                        get_sql_quoted_identifier("_origin"),
                        get_sql_quoted_identifier(name),
                    ])
                }
                models::ComparisonTarget::Column { name, path } => {
                    if !path.is_empty() {
                        todo!("comparison against other tables not supported")
                    }
                    Expr::CompoundIdentifier(vec![
                        get_sql_quoted_identifier("_origin"),
                        get_sql_quoted_identifier(name),
                    ])
                }
            };

            let right = match value {
                models::ComparisonValue::Column { .. } => {
                    todo!("Column comparison not supported")
                }
                models::ComparisonValue::Scalar { value } => {
                    match value {
                        serde_json::Value::Number(n) => Expr::Value(Value::Number(n.to_string(), true)),
                        serde_json::Value::String(s) => Expr::Value(Value::SingleQuotedString(s.to_string())),
                        _ => Expr::Value(Value::Placeholder(value.to_string()))
                    }
                }
                models::ComparisonValue::Variable { .. } => {
                    todo!("Not sure what variable comparison is")
                }
            };

            let operator = match operator {
                models::BinaryComparisonOperator::Equal => BinaryOperator::Eq,
                models::BinaryComparisonOperator::Other { name }  => {
                    // todo improve code
                    if name == &("like".to_string()) {
                        return get_sql_like_expr(left, right, false)
                    }
                    if name == &("nlike".to_string()) {
                        return get_sql_like_expr(left, right, true)
                    }
                    todo!("Only equality is supported");
                }
            };

            Expr::BinaryOp {
                left: Box::new(left),
                op: operator,
                right: Box::new(right),
            }
        }

        models::Expression::BinaryArrayComparisonOperator { .. } => todo!(),
        models::Expression::Exists { .. } => todo!(),
    }
}

/* Util functions for SQL query building */

// gets a quoted identifier to add in the SQL query
fn get_sql_quoted_identifier<S: Into<String>>(value: S) -> Ident {
    Ident::with_quote('"', value)
}

// self explanatory
fn get_sql_function_expression(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Function(Function {
        name: ObjectName(vec![Ident::new(name)]),
        args: args
            .into_iter()
            .map(|arg| FunctionArg::Unnamed(FunctionArgExpr::Expr(arg)))
            .collect(),
        over: None,
        distinct: false,
        special: false,
        order_by: vec![],
    })
}

// AND operator expression to be used in the predicate
pub fn get_sql_and_expression(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        left: Box::new(left),
        op: BinaryOperator::And,
        right: Box::new(right),
    }
}
// OR operator expression to be used in the predicate
pub fn get_sql_or_expr(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        left: Box::new(left),
        op: BinaryOperator::Or,
        right: Box::new(right),
    }
}
// LIKE operator expression to be used in the predicate
pub fn get_sql_like_expr(left: Expr, right: Expr, negated: bool) -> Expr {
    Expr::Like { negated: negated, expr: Box::new(left), pattern: Box::new(right), escape_char: None }
}
