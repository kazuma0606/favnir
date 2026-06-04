resource "snowflake_database" "demo" {
  name    = "DEMO_DB"
  comment = "Favnir e2e demo database"
}

resource "snowflake_schema" "public" {
  database = snowflake_database.demo.name
  name     = "PUBLIC"
}

resource "snowflake_table" "orders" {
  database = snowflake_database.demo.name
  schema   = snowflake_schema.public.name
  name     = "ORDERS"
  comment  = "Demo orders table for Favnir e2e pipeline"

  column {
    name     = "order_id"
    type     = "NUMBER(38,0)"
    nullable = false
  }
  column {
    name     = "customer"
    type     = "VARCHAR(256)"
    nullable = false
  }
  column {
    name     = "amount"
    type     = "FLOAT"
    nullable = false
  }
  column {
    name     = "region"
    type     = "VARCHAR(64)"
    nullable = false
  }
}
