# lambda-deploy — Favnir on AWS Lambda

This example shows how to deploy a Favnir pipeline to AWS Lambda using `fav deploy --target lambda`.

## Prerequisites

- [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2.html) configured (`aws configure`)
- An existing Lambda function (provided.al2 or provided.al2023 runtime)
- `fav` installed

## Steps (~30 min)

### 1. Create a Lambda function

```bash
aws lambda create-function \
  --function-name lambda-demo \
  --runtime provided.al2023 \
  --role arn:aws:iam::YOUR_ACCOUNT:role/lambda-execution-role \
  --handler bootstrap \
  --zip-file fileb://bootstrap.zip \
  --region ap-northeast-1
```

### 2. Build the native binary

```bash
fav build --target native
```

### 3. Package and deploy

```bash
# Package only (generates bootstrap.zip)
fav deploy --target lambda --package-only

# Package and deploy in one step
fav deploy --target lambda --function lambda-demo --region ap-northeast-1
```

### 4. Invoke

```bash
aws lambda invoke --function-name lambda-demo out.json
cat out.json
```

## Configuration via fav.toml

Set defaults in `fav.toml` so you can run `fav deploy --target lambda` without extra flags:

```toml
[deploy]
target = "lambda"
function = "lambda-demo"
region = "ap-northeast-1"
memory = 256
timeout = 30
```
