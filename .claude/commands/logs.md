Fetch recent logs from the rune-registry Lambda function.

```bash
aws logs tail /aws/lambda/favnir-registry \
  --region ap-northeast-1 \
  --since 1h \
  --format short
```

Show the last hour of logs. If $ARGUMENTS is provided, use it as the `--since` value (e.g. `30m`, `2h`, `1d`).

Also show any ERROR lines prominently at the top if found.
