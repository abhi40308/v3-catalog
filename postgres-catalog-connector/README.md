> This is a temp repo. This code will likely be moved somewhere else.

# PG Catalog Connector

A custom data connector that accepts PG database URL as an argument and returns the tables and columns outside `information_schema` and `pg_catalog` schema. This data connector is implemented looking at the [NDC specification](https://github.com/hasura/ndc-spec) and the referring to the [ndc-postgres connector](https://github.com/hasura/ndc-postgres).

### Development

1. Start server using `PORT=5000 cargo run`
2. Get schema: `curl http://localhost:5000/schema`
3. Get capabilities: `curl http://localhost:5000/capabilities`
4. Make queries:

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

##### Foreign keys query:

Foreign keys are fetched with a query equivalent to:

```
	SELECT
		q.schema_from AS schema_from,
		q.table_name AS table_from,
		q.fkey_name AS fkey_name,
		min(q.schema_to) AS schema_to,
		min(q.table_to) AS table_to,
		min(q.confupdtype) AS on_update,
		min(q.confdeltype) AS on_delete,
		json_object_agg(ac.attname, afc.attname) AS column_mapping

	FROM (
		SELECT
			ctn.nspname AS schema_from,
			ct.relname AS table_name,
			r.conrelid AS table_id,
			r.conname AS fkey_name,
			cftn.nspname AS schema_to,
			cft.relname AS table_to,
			r.confrelid AS ref_table_id,
			r.confupdtype,
			r.confdeltype,
			unnest(r.conkey) AS column_id,
			unnest(r.confkey) AS ref_column_id
		FROM
			pg_constraint AS r
			JOIN pg_class AS ct ON r.conrelid = ct.oid
			JOIN pg_namespace AS ctn ON ct.relnamespace = ctn.oid
			JOIN pg_class AS cft ON r.confrelid = cft.oid
			JOIN pg_namespace AS cftn ON cft.relnamespace = cftn.oid
    	WHERE
	      r.contype = 'f' AND
	      ctn.nspname NOT LIKE 'pg_%' AND
	      ctn.nspname NOT LIKE 'hdb_%'
	    ) AS q

	JOIN pg_attribute AS ac
	ON q.column_id = ac.attnum
	AND q.table_id = ac.attrelid

	JOIN pg_attribute afc
	ON q.ref_column_id = afc.attnum
	AND q.ref_table_id = afc.attrelid

	GROUP BY q.schema_from, q.table_name, q.fkey_name
```

The following curl command can be used to fetch foreign keys:

```
curl -d '{ "table": "foreign_keys", "query": { "limit": 10, "fields": { "fkey_name": { "type": "column", "column": "fkey_name", "arguments": {} }, "table_from": { "type": "column", "column": "table_from", "arguments": {} }, "schema_from": { "type": "column", "column": "schema_from", "arguments": {} }, "column_mapping": { "type": "column", "column": "column_mapping", "arguments": {} }, "on_update": { "type": "column", "column": "on_update", "arguments": {} }, "on_delete": { "type": "column", "column": "on_delete", "arguments": {} }, "table_to": { "type": "column", "column": "table_to", "arguments": {} }, "schema_to": { "type": "column", "column": "schema_to", "arguments": {} } } }, "arguments": { "database_url": {"type": "literal", "value": "postgres://postgres:postgrespassword@localhost:5432/postgres" } }, "table_relationships": {} }' -H "Content-Type: application/json" -X POST http://localhost:5500/query
```

Example Response of the above curl command:

```
[{"rows":[{"table_to":{"value":"owner"},"column_mapping":{"value":{"owner_id":"id"}},"fkey_name":{"value":"passport_info_owner_id_fkey"},"schema_from":{"value":"_onetoone"},"schema_to":{"value":"_onetoone"},"on_delete":{"value":"a"},"on_update":{"value":"a"},"table_from":{"value":"passport_info"}},{"on_delete":{"value":"a"},"table_to":{"value":"accounts"},"fkey_name":{"value":"sub_accounts_ref_num_ref_type_fkey"},"schema_to":{"value":"public"},"table_from":{"value":"sub_accounts"},"on_update":{"value":"a"},"schema_from":{"value":"_onetoone"},"column_mapping":{"value":{"ref_num":"acc_num","ref_type":"acc_type"}}}]}]
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
