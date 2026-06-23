/**
 * Stop hook for favnir:
 *   Plays a short beep when Claude finishes a task.
 *   Useful for long-running sessions where you've switched to another window.
 */
const { spawnSync } = require('child_process');

const chunks = [];
process.stdin.on('data', d => chunks.push(d));
process.stdin.on('end', () => {
  // Two-tone chime: low then high
  spawnSync('powershell.exe', [
    '-NoProfile', '-NonInteractive', '-Command',
    '[console]::beep(700,150); Start-Sleep -Milliseconds 80; [console]::beep(1000,200)',
  ], { timeout: 3000 });

  process.exit(0);
});
