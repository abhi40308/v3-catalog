# PG Catalog Connector

A custom data connector that accepts PG database URL as an argument and returns the tables and columns outside `information_schema` and `pg_catalog` schema. This data connector is implemented looking at the [NDC specification](https://github.com/hasura/ndc-spec) and the referring to the [ndc-postgres connector](https://github.com/hasura/ndc-postgres).

### Development

1. Start server using `PORT=5000 cargo run`
2. Get schema: `curl http://localhost:5000/schema`
3. Get capabilities: `curl http://localhost:5000/capabilities`
3. Make queries:
	- `curl -d '{ "table": "columns", "query": { "limit": 10, "fields": { "table_name": { "type": "column", "column": "table_name", "arguments": {} }, "column_name": { "type": "column", "column": "column_name", "arguments": {} }, "data_type": { "type": "column", "column": "data_type", "arguments": {} } } }, "arguments": { "database_url": {"type": "literal", "value": "postgres://test" } }, "table_relationships": {} }' -H "Content-Type: application/json" -X POST http://localhost:5000/query`

		- `curl -d '{ "table": "tables", "query": { "limit": 10, "fields": { "table_name": { "type": "column", "column": "table_name", "arguments": {}}}}, "arguments": { "database_url": {"type": "literal", "value": "" } }, "table_relationships": {} }' -H "Content-Type: application/json" -X POST http://localhost:5000/query`

### Capabilities

```
{
	"versions": "^1.0.0",
	"capabilities": {
		"query": {},
		"explain": {},
		"relationships": {}
	}
}
```

### How it works

This connector supports two tables: `tables` and `columns`.

Tables are fetched using a query like (not exactly this):

```
SELECT table_name, table_schema
FROM information_schema.tables
WHERE
	table_schema not like 'pg_%'
	AND
	table_schema != 'information_schema'
```

Columns are fetched using a query like (not exactly this):

```
SELECT table_name, table_schema
FROM information_schema.tables
WHERE
	table_schema not like 'pg_%'
	AND
	table_schema != 'information_schema'
```

### Schema

```
{
	"scalar_types": {
		"Int": {
			"aggregate_functions": {
				"min": {
					"result_type": {
						"type": "nullable",
						"underlying_type": {
							"type": "named",
							"name": "Int"
						}
					}
				},
				"max": {
					"result_type": {
						"type": "nullable",
						"underlying_type": {
							"type": "named",
							"name": "Int"
						}
					}
				}
			},
			"comparison_operators": {},
			"update_operators": {}
		},
		"String": {
			"aggregate_functions": {},
			"comparison_operators": {
				"like": {
					"argument_type": {
						"type": "named",
						"name": "String"
					}
				}
			},
			"update_operators": {}
		}
	},
	"object_types": {
		"columns": {
			"description": "Postgres column definition",
			"fields": {
				"column_name": {
					"description": "Name of the table column",
					"arguments": {},
					"type": {
						"type": "named",
						"name": "String"
					}
				},
				"table_name": {
					"description": "Name of the Postgres table",
					"arguments": {},
					"type": {
						"type": "named",
						"name": "String"
					}
				},
				"table": {
					"description": "Comment of the table column",
					"arguments": {},
					"type": {
						"type": "named",
						"name": "table"
					}
				},
				"table_schema": {
					"description": "Name of the schema of the Postgres table",
					"arguments": {},
					"type": {
						"type": "named",
						"name": "String"
					}
				},
				"comment": {
					"description": "Comment of the table column",
					"arguments": {},
					"type": {
						"type": "named",
						"name": "String"
					}
				}
			}
		},
		"tables": {
			"description": "Postgres table definition",
			"fields": {
				"columns": {
					"description": "The article's author ID",
					"arguments": {},
					"type": {
						"type": "array",
						"element_type": {
							"type": "named",
							"name": "column"
						}
					}
				},
				"comment": {
					"description": "Name of the Postgres table",
					"arguments": {},
					"type": {
						"type": "nullable",
						"underlying_type": {
							"type": "named",
							"name": "String"
						}
					}
				},
				"table_name": {
					"description": "Name of the Postgres table",
					"arguments": {},
					"type": {
						"type": "named",
						"name": "String"
					}
				},
				"table_schema": {
					"description": "Name of the schema of the Postgres table",
					"arguments": {},
					"type": {
						"type": "named",
						"name": "String"
					}
				}
			}
		}
	},
	"tables": [{
		"name": "tables",
		"description": "A collection of Postgres tables",
		"arguments": {
			"database_url": {
				"description": "The PG connection URI of the Postgres database that you wish to get entities from",
				"type": {
					"type": "named",
					"name": "database_url"
				}
			}
		},
		"type": "table",
		"deletable": false,
		"uniqueness_constraints": {
			"TableSchemaName": {
				"unique_columns": ["table_schema", "table_name"]
			}
		},
		"foreign_keys": {}
	}, {
		"name": "columns",
		"description": "A collection of Postgres columns",
		"arguments": {
			"database_url": {
				"description": "The PG connection URI of the Postgres database that you wish to get entities from",
				"type": {
					"type": "named",
					"name": "database_url"
				}
			}
		},
		"type": "column",
		"deletable": false,
		"uniqueness_constraints": {
			"ColumnName": {
				"unique_columns": ["table_schema", "table_name", "column_name"]
			}
		},
		"foreign_keys": {
			"ColumnToTable": {
				"column_mapping": {
					"table_name": "table_name",
					"table_schema": "table_schema"
				},
				"foreign_table": "table"
			}
		}
	}],
	"commands": []
}
```