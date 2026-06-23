/**
 * git-hooks.js — unified git pre-tool hook for Favnir
 *
 * Handles two gates in a single Node.js process (one startup per Bash call):
 *   git commit → cargo clippy -D warnings  (blocks on failure)
 *   git push   → cargo check --locked      (blocks on failure)
 *
 * Non-git commands exit immediately with minimal overhead.
 */
const { execSync, spawnSync } = require('child_process');

const chunks = [];
process.stdin.on('data', d => chunks.push(d));
process.stdin.on('end', () => {
  let input;
  try { input = JSON.parse(chunks.join('')); } catch { process.exit(0); }

  const cmd = ((input.tool_input && input.tool_input.command) || '').trimStart();

  const isCommit = /^git commit/.test(cmd);
  const isPush   = /^git push/.test(cmd);

  // Fast exit for non-git commands — no cargo invocation needed
  if (!isCommit && !isPush) process.exit(0);

  const repo = 'C:/Users/yoshi/favnir';

  // ── git commit: clippy ────────────────────────────────────────────────────
  if (isCommit) {
    // Auto-stage Cargo.lock if dirty
    try {
      const diff = execSync(`git -C "${repo}" diff --name-only -- fav/Cargo.lock`, { shell: true }).toString().trim();
      if (diff) {
        execSync(`git -C "${repo}" add fav/Cargo.lock`, { shell: true });
        process.stderr.write('[hook] auto-staged fav/Cargo.lock\n');
      }
    } catch (e) {
      process.stderr.write('[hook] Cargo.lock check failed: ' + e.message + '\n');
    }

    process.stderr.write('[hook] running cargo clippy...\n');
    const r = spawnSync('cargo', ['clippy', '--locked', '-j', '8', '--', '-D', 'warnings'], {
      cwd: repo + '/fav',
      shell: true,
      encoding: 'utf8',
      timeout: 120000,
    });

    if (r.status !== 0) {
      const errors = ((r.stdout || '') + (r.stderr || '')).split('\n')
        .filter(l => l.includes('error') || l.includes('-->'))
        .slice(0, 30).join('\n');
      process.stdout.write(JSON.stringify({
        continue: false,
        stopReason: `cargo clippy failed — fix before committing:\n\n${errors}`,
      }));
      process.exit(0);
    }

    process.stderr.write('[hook] clippy OK\n');
    process.exit(0);
  }

  // ── git push: cargo check ─────────────────────────────────────────────────
  if (isPush) {
    process.stderr.write('[hook] running cargo check before push...\n');
    const r = spawnSync('cargo', ['check', '--locked', '-j', '8'], {
      cwd: repo + '/fav',
      shell: true,
      encoding: 'utf8',
      timeout: 60000,
    });

    if (r.status !== 0) {
      const errors = ((r.stdout || '') + (r.stderr || '')).split('\n')
        .filter(l => l.startsWith('error') || l.includes('-->'))
        .slice(0, 20).join('\n');
      process.stdout.write(JSON.stringify({
        continue: false,
        stopReason: `cargo check failed — fix compile errors before pushing:\n\n${errors}`,
      }));
      process.exit(0);
    }

    process.stderr.write('[hook] cargo check OK\n');
    process.exit(0);
  }
});
