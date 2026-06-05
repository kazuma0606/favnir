# Amazon Linux 2023 の最新 AMI（ap-northeast-1）

data "aws_ami" "al2023" {
  most_recent = true
  owners      = ["amazon"]

  filter {
    name   = "name"
    values = ["al2023-ami-2023*-x86_64"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

# ── EC2 Instance ──────────────────────────────────────────────────────────────

resource "aws_instance" "favnir_ec2" {
  ami                    = data.aws_ami.al2023.id
  instance_type          = "t3.small"
  subnet_id              = aws_subnet.private_a.id
  vpc_security_group_ids = [aws_security_group.ec2.id]
  iam_instance_profile   = aws_iam_instance_profile.ec2_profile.name

  # user_data: バイナリ取得 → パイプライン実行 → 証跡 S3 保存
  user_data = <<-EOF
    #!/bin/bash
    set -euo pipefail
    exec > /var/log/favnir-airgap.log 2>&1

    BUCKET="${var.bucket_name}"
    TS=$(date +%Y%m%d-%H%M%S)
    PROOF=/tmp/proof-$TS.txt

    echo "=== Favnir Airgap E2E Demo ===" | tee $PROOF
    echo "Timestamp: $TS"                | tee -a $PROOF
    echo ""                              | tee -a $PROOF

    # ── Step 1: Favnir バイナリを /tmp/ にダウンロード（system パス変更なし）──

    echo "[Step 1] Downloading Favnir binary from S3..." | tee -a $PROOF
    aws s3 cp s3://$BUCKET/airgap/binary/fav /tmp/fav
    chmod +x /tmp/fav
    echo "Binary location: $(/tmp/fav --version 2>&1 | head -1)" | tee -a $PROOF

    # ── 証跡A: which fav → not found（システム PATH を汚染していない）────────

    echo ""                                                                     | tee -a $PROOF
    echo "[Proof] which fav: $(which fav 2>/dev/null || echo 'not found')"     | tee -a $PROOF
    echo "[Proof] /tmp/fav exists: $(ls -lh /tmp/fav)"                        | tee -a $PROOF
    echo "[Proof] No system install — binary confined to /tmp/"                | tee -a $PROOF

    # ── Step 2: analyze.fav + CSV データをダウンロード ───────────────────────

    echo ""                                                                     | tee -a $PROOF
    echo "[Step 2] Downloading pipeline and CSV data from S3..."               | tee -a $PROOF
    aws s3 cp s3://$BUCKET/airgap/src/analyze.fav /tmp/analyze.fav
    mkdir -p /tmp/data
    aws s3 cp s3://$BUCKET/airgap/data/txn_jan.csv /tmp/data/txn_jan.csv
    aws s3 cp s3://$BUCKET/airgap/data/txn_feb.csv /tmp/data/txn_feb.csv
    aws s3 cp s3://$BUCKET/airgap/data/txn_mar.csv /tmp/data/txn_mar.csv
    echo "Files ready: $(ls /tmp/data/)"                                       | tee -a $PROOF

    # ── Step 3: パイプライン実行 ─────────────────────────────────────────────

    echo ""                                                                     | tee -a $PROOF
    echo "[Step 3] Running Favnir ETL pipeline..."                             | tee -a $PROOF
    echo "---"                                                                 | tee -a $PROOF
    /tmp/fav run /tmp/analyze.fav \
      /tmp/data/txn_jan.csv \
      /tmp/data/txn_feb.csv \
      /tmp/data/txn_mar.csv 2>&1 | tee -a $PROOF
    echo "---"                                                                 | tee -a $PROOF

    # ── Step 4: 証跡を S3 にアップロード ────────────────────────────────────

    echo ""                                                                     | tee -a $PROOF
    echo "[Step 4] Uploading proof to S3..."                                   | tee -a $PROOF
    aws s3 cp $PROOF s3://$BUCKET/airgap/proof/proof-$TS.txt
    echo "[Done] Proof uploaded to s3://$BUCKET/airgap/proof/proof-$TS.txt"   | tee -a $PROOF
  EOF

  tags = { Name = "favnir-airgap-ec2" }
}
