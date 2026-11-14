/**
 * `npm run content:lint [-- <regex tên tệp>...] [--json] [--summary] [--fix] [--fail-on=high|medium|low|none]`
 * Dò thuật ngữ/cụm từ không chuẩn trong `seed/content/*.md` theo `seed/terminology.json`.
 * Mặc định chỉ đọc. `--fix` ghi đè tệp, chỉ áp các rule có `autofix`/`replace`, rồi báo những lỗi còn lại.
 * Ví dụ: `npm run content:lint -- ^system-testing ^unit-test --summary`
 */
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { SEVERITIES, applyFixes, compileRules, lintMarkdown, type Finding, type Severity, type Terminology } from '../seed/contentLint.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const contentDir = path.join(root, 'seed/content');
const args = process.argv.slice(2);
const flag = (name: string) => args.includes(`--${name}`);
const failOnArg = args.find((a) => a.startsWith('--fail-on='))?.split('=')[1] ?? 'high';
if (failOnArg !== 'none' && !SEVERITIES.includes(failOnArg as Severity)) {
  throw new Error(`--fail-on phải là high | medium | low | none, nhận "${failOnArg}".`);
}
const patterns = args.filter((a) => !a.startsWith('--')).map((p) => new RegExp(p));

const terminology = JSON.parse(readFileSync(path.join(root, 'seed/terminology.json'), 'utf8')) as Terminology;
const rules = compileRules(terminology);
const files = readdirSync(contentDir)
  .filter((f) => f.endsWith('.md'))
  .filter((f) => patterns.length === 0 || patterns.some((p) => p.test(f)))
  .sort();

let fixed = 0;
const findings: Finding[] = files.flatMap((f) => {
  const file = path.join(contentDir, f);
  const raw = readFileSync(file, 'utf8');
  const found = lintMarkdown(f, raw, rules);
  if (!flag('fix') || !found.some((x) => x.fix !== undefined)) return found;
  const { output, applied } = applyFixes(raw, found);
  if (applied.length === 0) return found;
  writeFileSync(file, output, 'utf8');
  fixed += applied.length;
  return lintMarkdown(f, output, rules);
});
if (flag('fix')) console.error(`Đã tự sửa ${fixed} chỗ.`);

if (flag('json')) {
  process.stdout.write(JSON.stringify({ files: files.length, findings }, null, 2) + '\n');
} else {
  if (!flag('summary')) {
    let current = '';
    for (const x of findings) {
      if (x.file !== current) {
        current = x.file;
        console.log(`\n${x.file}`);
      }
      console.log(`  ${x.line}:${x.column}  [${x.severity}] ${x.ruleId}: "${x.match}" → "${x.suggest}"${x.note ? ` (${x.note})` : ''}`);
      console.log(`      ${x.excerpt}`);
    }
  }
  const bySeverity = Object.fromEntries(SEVERITIES.map((s) => [s, findings.filter((x) => x.severity === s).length]));
  const byRule = new Map<string, number>();
  for (const x of findings) byRule.set(x.ruleId, (byRule.get(x.ruleId) ?? 0) + 1);
  const affected = new Set(findings.map((x) => x.file)).size;
  console.log(`\n${files.length} tệp, ${affected} tệp có lỗi, ${findings.length} lỗi — high ${bySeverity.high}, medium ${bySeverity.medium}, low ${bySeverity.low}`);
  for (const [id, n] of [...byRule].sort((a, b) => b[1] - a[1]).slice(0, 20)) console.log(`  ${String(n).padStart(5)}  ${id}`);
}

if (failOnArg !== 'none') {
  const limit = SEVERITIES.indexOf(failOnArg as Severity);
  if (findings.some((x) => SEVERITIES.indexOf(x.severity) <= limit)) process.exitCode = 1;
}
