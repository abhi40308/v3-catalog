use indexmap::IndexMap;
use ndc_client::models::{self};
use sqlparser::ast::{
    BinaryOperator, Expr, Ident, Join, JoinConstraint, JoinOperator, ObjectName, Query, SelectItem,
    TableAlias, TableFactor, TableWithJoins, Value,
};

use crate::error::ServerError;
use crate::tables::SupportedCollection;

use crate::sql::predicate_builder::get_predicate_expression;
use crate::sql::utils::{
    get_equivalent_table_column, get_sql_and_expression, get_sql_eq_expression,
    get_sql_function_expression, get_sql_like_expr, get_sql_query, get_sql_quoted_identifier,
};

pub fn get_fkey_query(
    query: &ndc_client::models::Query,
    table: &SupportedCollection,
) -> Result<Box<Query>, ServerError> {
    // Builds the Select clause. This is the equivalent SQL statement (if all fields are selected)
    // SELECT q.schema_from AS schema_from,
    // q.table_name AS table_from,
    // q.fkey_name AS fkey_name,
    // min(q.schema_to) AS schema_to,
    // min(q.table_to) AS table_to,
    // min(q.confupdtype) AS on_update,
    // min(q.confdeltype) AS on_delete,
    // json_object_agg(ac.attname, afc.attname) AS column_mapping

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

                                match &column_info.name[..] {
                                    "column_mapping" => get_sql_function_expression(
                                        "json_object_agg",
                                        vec![
                                            Expr::CompoundIdentifier(vec![
                                                get_sql_quoted_identifier("ac"),
                                                get_sql_quoted_identifier("attname"),
                                            ]),
                                            Expr::CompoundIdentifier(vec![
                                                get_sql_quoted_identifier("afc"),
                                                get_sql_quoted_identifier("attname"),
                                            ]),
                                        ],
                                        None,
                                    ),
                                    "schema_to" | "table_to" | "on_update" | "on_delete" => {
                                        get_sql_function_expression(
                                            "min",
                                            vec![Expr::CompoundIdentifier(vec![
                                                get_sql_quoted_identifier("q"),
                                                get_sql_quoted_identifier(
                                                    get_equivalent_table_column(&column_info.name),
                                                ),
                                            ])],
                                            None,
                                        )
                                    }
                                    "schema_from" | "table_from" | "fkey_name" => {
                                        Expr::CompoundIdentifier(vec![
                                            get_sql_quoted_identifier("q"),
                                            get_sql_quoted_identifier(get_equivalent_table_column(
                                                &column_info.name,
                                            )),
                                        ])
                                    }
                                    _ => todo!(),
                                }
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

    // Builds from clause. This is the SQL equivalent from clause with the joins:
    // From (...subquery) AS q
    //  JOIN pg_attribute AS ac
    //  ON q.column_id = ac.attnum
    //  AND q.table_id = ac.attrelid
    //  JOIN pg_attribute afc
    //  ON q.ref_column_id = afc.attnum
    //  AND q.ref_table_id = afc.attrelid

    let fkey_subquery = get_fkey_subquery();
    let rows_from = vec![TableWithJoins {
        relation: TableFactor::Derived {
            lateral: false,
            subquery: fkey_subquery,
            alias: Some(TableAlias {
                columns: vec![],
                name: get_sql_quoted_identifier("q"),
            }),
        },
        joins: vec![
            Join {
                relation: TableFactor::Table {
                    name: ObjectName(vec![get_sql_quoted_identifier("pg_attribute")]),
                    alias: Some(TableAlias {
                        name: get_sql_quoted_identifier("ac"),
                        columns: vec![],
                    }),
                    args: None,
                    with_hints: vec![],
                },
                join_operator: JoinOperator::Inner(JoinConstraint::On(get_sql_and_expression(
                    get_sql_eq_expression(
                        Expr::CompoundIdentifier(vec!["q".into(), "column_id".into()]),
                        Expr::CompoundIdentifier(vec!["ac".into(), "attnum".into()]),
                    ),
                    get_sql_eq_expression(
                        Expr::CompoundIdentifier(vec!["q".into(), "table_id".into()]),
                        Expr::CompoundIdentifier(vec!["ac".into(), "attrelid".into()]),
                    ),
                ))),
            },
            Join {
                relation: TableFactor::Table {
                    name: ObjectName(vec![get_sql_quoted_identifier("pg_attribute")]),
                    alias: Some(TableAlias {
                        name: get_sql_quoted_identifier("afc"),
                        columns: vec![],
                    }),
                    args: None,
                    with_hints: vec![],
                },
                join_operator: JoinOperator::Inner(JoinConstraint::On(get_sql_and_expression(
                    get_sql_eq_expression(
                        Expr::CompoundIdentifier(vec!["q".into(), "ref_column_id".into()]),
                        Expr::CompoundIdentifier(vec!["afc".into(), "attnum".into()]),
                    ),
                    get_sql_eq_expression(
                        Expr::CompoundIdentifier(vec!["q".into(), "ref_table_id".into()]),
                        Expr::CompoundIdentifier(vec!["afc".into(), "attrelid".into()]),
                    ),
                ))),
            },
        ],
    }];

    // append the predicate coming from the query
    let predicate: Option<Expr> = match &query.predicate {
        Some(p) => Some(get_predicate_expression(p, "q")),
        None => None,
    };

    // Group by clause, equivalent sql query is:
    // GROUP BY q.schema_from, q.table_name, q.fkey_name
    let group_by = vec![
        Expr::CompoundIdentifier(vec!["q".into(), "schema_from".into()]),
        Expr::CompoundIdentifier(vec!["q".into(), "table_name".into()]),
        Expr::CompoundIdentifier(vec!["q".into(), "fkey_name".into()]),
    ];

    Ok(get_sql_query(
        rows_projection,
        rows_from,
        predicate,
        Some(group_by),
        query.limit,
        query.offset,
    ))
}

fn get_fkey_subquery() -> Box<Query> {
    // Select statement, equivalent sql is:
    // SELECT
    // 		ctn.nspname AS schema_from,
    // 		ct.relname AS table_name,
    // 		r.conrelid AS table_id,
    // 		r.conname AS fkey_name,
    // 		cftn.nspname AS schema_to,
    // 		cft.relname AS table_to,
    // 		r.confrelid AS ref_table_id,
    // 		r.confupdtype,
    // 		r.confdeltype,
    // 		unnest(r.conkey) AS column_id,
    // 		unnest(r.confkey) AS ref_column_id

    let rows_projection = vec![
        SelectItem::ExprWithAlias {
            expr: Expr::CompoundIdentifier(vec![
                get_sql_quoted_identifier("ctn"),
                get_sql_quoted_identifier("nspname"),
            ]),
            alias: get_sql_quoted_identifier("schema_from"),
        },
        SelectItem::ExprWithAlias {
            expr: Expr::CompoundIdentifier(vec![
                get_sql_quoted_identifier("ct"),
                get_sql_quoted_identifier("relname"),
            ]),
            alias: get_sql_quoted_identifier("table_name"),
        },
        SelectItem::ExprWithAlias {
            expr: Expr::CompoundIdentifier(vec![
                get_sql_quoted_identifier("r"),
                get_sql_quoted_identifier("conrelid"),
            ]),
            alias: get_sql_quoted_identifier("table_id"),
        },
        SelectItem::ExprWithAlias {
            expr: Expr::CompoundIdentifier(vec![
                get_sql_quoted_identifier("r"),
                get_sql_quoted_identifier("conname"),
            ]),
            alias: get_sql_quoted_identifier("fkey_name"),
        },
        SelectItem::ExprWithAlias {
            expr: Expr::CompoundIdentifier(vec![
                get_sql_quoted_identifier("cftn"),
                get_sql_quoted_identifier("nspname"),
            ]),
            alias: get_sql_quoted_identifier("schema_to"),
        },
        SelectItem::ExprWithAlias {
            expr: Expr::CompoundIdentifier(vec![
                get_sql_quoted_identifier("cft"),
                get_sql_quoted_identifier("relname"),
            ]),
            alias: get_sql_quoted_identifier("table_to"),
        },
        SelectItem::ExprWithAlias {
            expr: Expr::CompoundIdentifier(vec![
                get_sql_quoted_identifier("r"),
                get_sql_quoted_identifier("confrelid"),
            ]),
            alias: get_sql_quoted_identifier("ref_table_id"),
        },
        SelectItem::UnnamedExpr(Expr::CompoundIdentifier(vec![
            get_sql_quoted_identifier("r"),
            get_sql_quoted_identifier("confupdtype"),
        ])),
        SelectItem::UnnamedExpr(Expr::CompoundIdentifier(vec![
            get_sql_quoted_identifier("r"),
            get_sql_quoted_identifier("confdeltype"),
        ])),
        SelectItem::ExprWithAlias {
            expr: get_sql_function_expression(
                "unnest",
                vec![Expr::CompoundIdentifier(vec![
                    get_sql_quoted_identifier("r"),
                    get_sql_quoted_identifier("conkey"),
                ])],
                None,
            ),
            alias: get_sql_quoted_identifier("column_id"),
        },
        SelectItem::ExprWithAlias {
            expr: get_sql_function_expression(
                "unnest",
                vec![Expr::CompoundIdentifier(vec![
                    get_sql_quoted_identifier("r"),
                    get_sql_quoted_identifier("confkey"),
                ])],
                None,
            ),
            alias: get_sql_quoted_identifier("ref_column_id"),
        },
    ];

    // Builds from statement, the equivalent sql query looks like this:
    // FROM pg_constraint AS r
    // JOIN pg_class ct ON r.conrelid = ct.oid
    // JOIN pg_namespace ctn ON ct.relnamespace = ctn.oid
    // JOIN pg_class cft ON r.confrelid = cft.oid
    // JOIN pg_namespace cftn ON cft.relnamespace = cftn.oid

    let rows_from = vec![TableWithJoins {
        joins: vec![
            Join {
                relation: TableFactor::Table {
                    name: ObjectName(vec![get_sql_quoted_identifier("pg_class")]),
                    alias: Some(TableAlias {
                        name: get_sql_quoted_identifier("ct"),
                        columns: vec![],
                    }),
                    args: None,
                    with_hints: vec![],
                },
                join_operator: JoinOperator::Inner(JoinConstraint::On(Expr::BinaryOp {
                    left: Box::new(Expr::CompoundIdentifier(vec![
                        "r".into(),
                        "conrelid".into(),
                    ])),
                    op: (BinaryOperator::Eq),
                    right: Box::new(Expr::CompoundIdentifier(vec!["ct".into(), "oid".into()])),
                })),
            },
            Join {
                relation: TableFactor::Table {
                    name: ObjectName(vec![get_sql_quoted_identifier("pg_namespace")]),
                    alias: Some(TableAlias {
                        name: get_sql_quoted_identifier("ctn"),
                        columns: vec![],
                    }),
                    args: None,
                    with_hints: vec![],
                },
                join_operator: JoinOperator::Inner(JoinConstraint::On(Expr::BinaryOp {
                    left: Box::new(Expr::CompoundIdentifier(vec![
                        "ct".into(),
                        "relnamespace".into(),
                    ])),
                    op: (BinaryOperator::Eq),
                    right: Box::new(Expr::CompoundIdentifier(vec!["ctn".into(), "oid".into()])),
                })),
            },
            Join {
                relation: TableFactor::Table {
                    name: ObjectName(vec![get_sql_quoted_identifier("pg_class")]),
                    alias: Some(TableAlias {
                        name: get_sql_quoted_identifier("cft"),
                        columns: vec![],
                    }),
                    args: None,
                    with_hints: vec![],
                },
                join_operator: JoinOperator::Inner(JoinConstraint::On(Expr::BinaryOp {
                    left: Box::new(Expr::CompoundIdentifier(vec![
                        "r".into(),
                        "confrelid".into(),
                    ])),
                    op: (BinaryOperator::Eq),
                    right: Box::new(Expr::CompoundIdentifier(vec!["cft".into(), "oid".into()])),
                })),
            },
            Join {
                relation: TableFactor::Table {
                    name: ObjectName(vec![get_sql_quoted_identifier("pg_namespace")]),
                    alias: Some(TableAlias {
                        name: get_sql_quoted_identifier("cftn"),
                        columns: vec![],
                    }),
                    args: None,
                    with_hints: vec![],
                },
                join_operator: JoinOperator::Inner(JoinConstraint::On(Expr::BinaryOp {
                    left: Box::new(Expr::CompoundIdentifier(vec![
                        "cft".into(),
                        "relnamespace".into(),
                    ])),
                    op: (BinaryOperator::Eq),
                    right: Box::new(Expr::CompoundIdentifier(vec!["cftn".into(), "oid".into()])),
                })),
            },
        ],
        relation: TableFactor::Table {
            name: ObjectName(vec![get_sql_quoted_identifier("pg_constraint")]),
            alias: Some(TableAlias {
                name: get_sql_quoted_identifier("r"),
                columns: vec![],
            }),
            args: None,
            with_hints: vec![],
        },
    }];

    // Builds the where condition. Equivalent sql query is :
    // WHERE r.contype = 'f' AND
    // ctn.nspname NOT LIKE 'pg_%' AND
    // ctn.nspname NOT LIKE 'hdb_%'
    let predicate = get_sql_and_expression(
        get_sql_eq_expression(
            Expr::CompoundIdentifier(vec!["r".into(), "contype".into()]),
            Expr::Identifier(Ident::with_quote('\'', "f")),
        ),
        get_sql_and_expression(
            get_sql_like_expr(
                Expr::CompoundIdentifier(vec!["ctn".into(), "nspname".into()]),
                Expr::Identifier(Ident::with_quote('\'', "pg_%")),
                true,
            ),
            get_sql_like_expr(
                Expr::CompoundIdentifier(vec!["ctn".into(), "nspname".into()]),
                Expr::Identifier(Ident::with_quote('\'', "hdb_%")),
                true,
            ),
        ),
    );

    get_sql_query(
        rows_projection,
        rows_from,
        Some(predicate),
        None,
        None,
        None,
    )
}
