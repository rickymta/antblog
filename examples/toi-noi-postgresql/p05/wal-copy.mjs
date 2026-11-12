// archive_command/restore_command: no shell expansion, refuse conflicting archives.
import { copyFileSync, existsSync, readFileSync, constants } from 'node:fs';
const [mode, source, target] = process.argv.slice(2);
try {
  if (!['archive', 'restore'].includes(mode) || !source || !target) throw Error('arguments');
  if (mode === 'archive' && existsSync(target)) {
    if (!readFileSync(source).equals(readFileSync(target))) throw Error('archive conflict');
  } else copyFileSync(source, target, mode === 'archive' ? constants.COPYFILE_EXCL : 0);
} catch { process.exitCode = 1; }
