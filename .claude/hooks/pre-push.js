/**
 * Pre-push hook for favnir:
 *   Runs `cargo check --locked` before `git push` to catch compile errors
 *   that might have slipped through without a full test run.
 *   (Full tests are too slow to run on every push; use /test manually.)
 */
const { spawnSync } = require('child_process');

const chunks = [];
process.stdin.on('data', d => chunks.push(d));
process.stdin.on('end', () => {
  let input;
  try { input = JSON.parse(chunks.join('')); } catch { process.exit(0); }

  const cmd = (input.tool_input && input.tool_input.command) || '';
  if (!/^git push/.test(cmd)) process.exit(0);

  const repo = 'C:/Users/yoshi/favnir';

  process.stderr.write('[hook] running cargo check before push...\n');
  const r = spawnSync('cargo', ['check', '--locked', '-j', '8'], {
    cwd: repo + '/fav',
    shell: true,
    encoding: 'utf8',
    timeout: 60000,
  });

  if (r.status !== 0) {
    const out = ((r.stdout || '') + (r.stderr || '')).trim();
    const errors = out.split('\n')
      .filter(l => l.startsWith('error') || l.includes('-->'))
      .slice(0, 20)
      .join('\n');
    process.stdout.write(JSON.stringify({
      continue: false,
      stopReason: `cargo check failed — fix compile errors before pushing:\n\n${errors}\n\nRun /build check to see all errors.`,
    }));
    process.exit(0);
  }

  process.stderr.write('[hook] cargo check OK — proceeding with push\n');
  process.exit(0);
});
