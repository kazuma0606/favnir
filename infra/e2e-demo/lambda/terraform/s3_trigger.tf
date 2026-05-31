# ---- S3 → Lambda Compiler トリガー ----

# S3 が Lambda を invoke する権限
resource "aws_lambda_permission" "s3_invoke_compiler" {
  statement_id  = "AllowS3Invoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.compiler.function_name
  principal     = "s3.amazonaws.com"
  source_arn    = "arn:aws:s3:::${var.bucket_name}"
}

# S3 バケットのイベント通知（source/*.fav が投入されたら compiler を起動）
resource "aws_s3_bucket_notification" "compiler_trigger" {
  bucket = var.bucket_name

  lambda_function {
    lambda_function_arn = aws_lambda_function.compiler.arn
    events              = ["s3:ObjectCreated:*"]
    filter_prefix       = "source/"
    filter_suffix       = ".fav"
  }

  depends_on = [aws_lambda_permission.s3_invoke_compiler]
}
