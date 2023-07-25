use sqlparser::ast::{
    BinaryOperator, Expr, Function, FunctionArg, FunctionArgExpr, Ident, ObjectName, Offset, Query,
    Select, SelectItem, SetExpr, TableWithJoins, Value,
};

// Gets a quoted identifier to add in the SQL query
pub fn get_sql_quoted_identifier<S: Into<String>>(value: S) -> Ident {
    Ident::with_quote('"', value)
}

// Get equivalent table entities using field names for fkey query
pub fn get_equivalent_table_column(x: &str) -> &str {
    match x {
        "schema_from" => "schema_from",
        "table_from" => "table_name",
        "fkey_name" => "fkey_name",
        "schema_to" => "schema_to",
        "table_to" => "table_to",
        "on_update" => "confupdtype",
        "on_delete" => "confdeltype",
        _ => todo!(),
    }
}

// Builds sql function expression
pub fn get_sql_function_expression(name: &str, args: Vec<Expr>, distinct: Option<bool>) -> Expr {
    Expr::Function(Function {
        name: ObjectName(vec![Ident::new(name)]),
        args: args
            .into_iter()
            .map(|arg| FunctionArg::Unnamed(FunctionArgExpr::Expr(arg)))
            .collect(),
        over: None,
        distinct: distinct.unwrap_or(false),
        special: false,
        order_by: vec![],
    })
}

// Eq operator expression to be used in the predicate
pub fn get_sql_eq_expression(left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        left: Box::new(left),
        op: BinaryOperator::Eq,
        right: Box::new(right),
    }
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
    Expr::Like {
        negated: negated,
        expr: Box::new(left),
        pattern: Box::new(right),
        escape_char: None,
    }
}

// util function to build an SQL query from constructed parameters
pub fn get_sql_query(
    projection: Vec<SelectItem>,
    from: Vec<TableWithJoins>,
    predicate: Option<Expr>,
    group_by: Option<Vec<Expr>>,
    limit: Option<u32>,
    offset: Option<u32>,
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
            group_by: group_by.unwrap_or(vec![]),
            cluster_by: vec![],
            distribute_by: vec![],
            sort_by: vec![],
            having: None,
            qualify: None,
            named_window: vec![],
        }))),
        limit: limit.map(|l| Expr::Value(Value::Number(l.to_string(), false))),
        offset: offset.map(|o| Offset {
            value: Expr::Value(Value::Number(o.to_string(), false)),
            rows: sqlparser::ast::OffsetRows::None,
        }),
        fetch: None,
        locks: vec![],
        order_by: vec![], //todo
    })
}
