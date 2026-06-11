/**
 * Pre-commit hook for favnir:
 *   1. Auto-stage fav/Cargo.lock if it has unstaged changes
 *   2. Run `cargo clippy --locked -- -D warnings` in fav/
 *      If clippy fails → block the commit with the error output
 */
const { execSync, spawnSync } = require('child_process');

const chunks = [];
process.stdin.on('data', d => chunks.push(d));
process.stdin.on('end', () => {
  let input;
  try { input = JSON.parse(chunks.join('')); } catch { process.exit(0); }

  const cmd = (input.tool_input && input.tool_input.command) || '';
  if (!/^git commit/.test(cmd)) process.exit(0);

  const repo = 'C:/Users/yoshi/favnir';

  // 1. Auto-stage Cargo.lock
  try {
    const diff = execSync(`git -C "${repo}" diff --name-only -- fav/Cargo.lock`, { shell: true }).toString().trim();
    if (diff) {
      execSync(`git -C "${repo}" add fav/Cargo.lock`, { shell: true });
      process.stderr.write('[hook] auto-staged fav/Cargo.lock\n');
    }
  } catch (e) {
    process.stderr.write('[hook] Cargo.lock check failed: ' + e.message + '\n');
  }

  // 2. Run clippy
  process.stderr.write('[hook] running cargo clippy...\n');
  const r = spawnSync('cargo', ['clippy', '--locked', '--', '-D', 'warnings'], {
    cwd: repo + '/fav',
    shell: true,
    encoding: 'utf8',
    timeout: 120000,
  });

  if (r.status !== 0) {
    const out = ((r.stdout || '') + (r.stderr || '')).trim();
    // Extract only error lines to keep the message concise
    const errors = out.split('\n')
      .filter(l => l.includes('error') || l.includes('-->'))
      .slice(0, 30)
      .join('\n');
    process.stdout.write(JSON.stringify({
      continue: false,
      stopReason: `cargo clippy failed — fix before committing:\n\n${errors}`,
    }));
    process.exit(0);
  }

  process.stderr.write('[hook] clippy OK\n');
  process.exit(0);
});
