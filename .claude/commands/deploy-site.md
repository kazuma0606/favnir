Build and deploy the Favnir reference site to S3 + CloudFront.

Steps:
1. Build the Next.js site:
```bash
cd /c/Users/yoshi/favnir/site && npm run build
```

2. Sync to S3:
```bash
# HTML/JSON: no cache
aws s3 sync out/ s3://favnir-site \
  --exclude "_next/*" --exclude "wasm/*" \
  --cache-control "max-age=0, must-revalidate" \
  --delete

# JS/CSS assets: immutable cache
aws s3 sync out/_next/ s3://favnir-site/_next/ \
  --cache-control "max-age=31536000, immutable"

# WASM files: must-revalidate
aws s3 sync out/wasm/ s3://favnir-site/wasm/ \
  --cache-control "max-age=0, must-revalidate"
```

3. Invalidate CloudFront:
```bash
aws cloudfront create-invalidation \
  --distribution-id E3KPK4T7Y5ZBDA \
  --paths "/*"
```

Site URL: https://dyrlmlnmak6gl.cloudfront.net

Report success or errors at each step.
