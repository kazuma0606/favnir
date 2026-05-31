resource "aws_db_subnet_group" "demo" {
  name       = "favnir-demo"
  subnet_ids = [aws_subnet.private.id, aws_subnet.private_b.id]
  tags       = { Name = "favnir-demo" }
}

# Aurora Serverless v2 は複数 AZ の subnet group が必要なため 2 つ目の subnet を追加
resource "aws_subnet" "private_b" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.0.3.0/24"
  availability_zone = "${var.aws_region}c"
  tags              = { Name = "favnir-private-b" }
}

resource "aws_rds_cluster" "demo" {
  cluster_identifier      = "favnir-demo"
  engine                  = "aurora-postgresql"
  engine_mode             = "provisioned"
  engine_version          = "16.13"
  database_name           = "demo"
  master_username         = var.db_user
  master_password         = var.db_password
  db_subnet_group_name    = aws_db_subnet_group.demo.name
  vpc_security_group_ids  = [aws_security_group.rds.id]
  skip_final_snapshot     = true

  serverlessv2_scaling_configuration {
    min_capacity = 0.5
    max_capacity = 1.0
  }
}

resource "aws_rds_cluster_instance" "demo" {
  identifier         = "favnir-demo-instance"
  cluster_identifier = aws_rds_cluster.demo.id
  instance_class     = "db.serverless"
  engine             = aws_rds_cluster.demo.engine
  engine_version     = aws_rds_cluster.demo.engine_version
}
