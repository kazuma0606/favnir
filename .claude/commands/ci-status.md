Check the status of GitHub Actions CI workflows for the Favnir repository.

```bash
cd /c/Users/yoshi/favnir && gh run list --limit 8 --json status,conclusion,name,headBranch,createdAt,url \
  | jq -r '.[] | "\(.status)\t\(.conclusion // "-")\t\(.name)\t\(.headBranch)\t\(.createdAt[:16])\t\(.url)"' \
  | column -t -s $'\t'
```

Then check if any workflow is currently running:
```bash
cd /c/Users/yoshi/favnir && gh run list --limit 3 --status in_progress --json name,url \
  | jq -r '.[] | "RUNNING: \(.name) — \(.url)"'
```

Report format:
- Show a table of recent runs with status (✓ success / ✗ failure / ⏳ in_progress)
- Highlight any failures prominently
- If $ARGUMENTS is a run ID, show detailed logs: `gh run view $ARGUMENTS --log-failed`

If all recent runs are green: 「CI 全グリーン」
If any failure: show the failed workflow name and a link to the logs.
