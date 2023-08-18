use ndc_client::models::{self};
use sqlparser::ast::Expr;
use sqlparser::ast::{BinaryOperator, UnaryOperator, Value};

use crate::sql::utils::{
    get_sql_and_expression, get_sql_like_expr, get_sql_or_expr, get_sql_quoted_identifier,
};

// builds a predicate expression as expected by the sqlx client
pub fn get_predicate_expression(expr: &models::Expression, alias: &str) -> Expr {
    match expr {
        models::Expression::And { expressions } => expressions
            .iter()
            .map(|e| get_predicate_expression(e, alias))
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
            .map(|e| get_predicate_expression(e, alias))
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
            expr: Box::new(get_predicate_expression(expression, alias)),
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
                models::ComparisonTarget::RootCollectionColumn { name } => {
                    Expr::CompoundIdentifier(vec![
                        get_sql_quoted_identifier(alias),
                        get_sql_quoted_identifier(name),
                    ])
                }
                models::ComparisonTarget::Column { name, path } => {
                    if !path.is_empty() {
                        todo!("comparison against other tables not supported")
                    }
                    Expr::CompoundIdentifier(vec![
                        get_sql_quoted_identifier(alias),
                        get_sql_quoted_identifier(name),
                    ])
                }
            };

            let right = match value {
                models::ComparisonValue::Column { .. } => {
                    todo!("Column comparison not supported")
                }
                models::ComparisonValue::Scalar { value } => match value {
                    serde_json::Value::Number(n) => Expr::Value(Value::Number(n.to_string(), true)),
                    serde_json::Value::String(s) => {
                        Expr::Value(Value::SingleQuotedString(s.to_string()))
                    }
                    _ => Expr::Value(Value::Placeholder(value.to_string())),
                },
                models::ComparisonValue::Variable { .. } => {
                    todo!("Not sure what variable comparison is")
                }
            };

            let operator = match operator {
                models::BinaryComparisonOperator::Equal => BinaryOperator::Eq,
                models::BinaryComparisonOperator::Other { name } => {
                    // todo improve code
                    if name == &("like".to_string()) {
                        return get_sql_like_expr(left, right, false);
                    }
                    if name == &("nlike".to_string()) {
                        return get_sql_like_expr(left, right, true);
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
