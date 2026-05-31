# ---- RDS Aurora PostgreSQL ----

resource "aws_rds_cluster" "demo" {
  cluster_identifier      = "favnir-lambda-demo"
  engine                  = "aurora-postgresql"
  engine_mode             = "provisioned"
  engine_version          = "16.6"
  database_name           = "favnirdb"
  master_username         = var.db_user
  master_password         = var.db_password
  db_subnet_group_name    = aws_db_subnet_group.demo.name
  vpc_security_group_ids  = [aws_security_group.rds.id]
  skip_final_snapshot     = true
  deletion_protection     = false
  backup_retention_period = 1
  enable_http_endpoint    = true

  serverlessv2_scaling_configuration {
    min_capacity = 0.5
    max_capacity = 1.0
  }

  tags = { Name = "favnir-lambda-demo" }
}

resource "aws_rds_cluster_instance" "demo" {
  identifier         = "favnir-lambda-demo-instance"
  cluster_identifier = aws_rds_cluster.demo.id
  instance_class     = "db.serverless"
  engine             = aws_rds_cluster.demo.engine
  engine_version     = aws_rds_cluster.demo.engine_version

  tags = { Name = "favnir-lambda-demo" }
}
