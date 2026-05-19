Build and deploy the rune-registry Lambda service to AWS.

Steps:
1. Build and push Docker image to ECR:
```bash
cd /c/Users/yoshi/favnir
AWS_ACCOUNT=847333136058
REGION=ap-northeast-1
ECR_URL=$AWS_ACCOUNT.dkr.ecr.$REGION.amazonaws.com/favnir-registry
SHA=$(git rev-parse --short HEAD)

aws ecr get-login-password --region $REGION | docker login --username AWS --password-stdin $AWS_ACCOUNT.dkr.ecr.$REGION.amazonaws.com

docker buildx build --provenance=false -f rune-registry/Dockerfile -t $ECR_URL:$SHA -t $ECR_URL:latest --push .
```

2. Update Lambda function code:
```bash
aws lambda update-function-code \
  --function-name favnir-registry \
  --image-uri $ECR_URL:$SHA \
  --region ap-northeast-1

aws lambda wait function-updated \
  --function-name favnir-registry \
  --region ap-northeast-1
```

3. Verify deployment with a quick health check:
```bash
curl -s https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com/runes
```

Report success or any errors at each step.
