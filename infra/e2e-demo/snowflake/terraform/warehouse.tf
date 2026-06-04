resource "snowflake_warehouse" "demo" {
  name           = "DEMO_WH"
  warehouse_size = "X-SMALL"
  auto_suspend   = 60
  auto_resume    = true
  comment        = "Favnir e2e demo warehouse"
}
