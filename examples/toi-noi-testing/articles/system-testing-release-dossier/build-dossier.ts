/**
 * SYS-17 · Dựng release dossier cho commit hiện tại (ứng viên).
 *   npx tsx articles/system-testing-release-dossier/build-dossier.ts [--gate]
 * 1. Chạy lại các suite Vitest của series trên ứng viên (JSON reporter).
 * 2. Ghép bằng chứng từ báo cáo (Playwright, đo hiệu năng, drill, phần cần con người) kèm commit đã ghi báo cáo.
 * 3. Kiểm độ mới bằng git diff theo watchPaths, áp luật trong rules.ts.
 * 4. Ghi reports/release-dossier.{json,md} (thư mục reports/ bị git ignore). --gate: exit 1 nếu chưa READY_FOR_SIGN_OFF/GO.
 * Script không ký thay ai: không có sign-off thì quyết định cao nhất là READY_FOR_SIGN_OFF.
 */
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { REGISTRY } from './registry';
import { assess, type Candidate, type EvidenceItem } from './rules';

const LAB_ROOT = fileURLToPath(new URL('../../', import.meta.url));
const git = (...args: string[]) => execFileSync('git', args, { cwd: LAB_ROOT, encoding: 'utf8' }).trim();
const REPO_ROOT = git('rev-parse', '--show-toplevel');
const gitRoot = (...args: string[]) => execFileSync('git', args, { cwd: REPO_ROOT, encoding: 'utf8' }).trim();

const head = gitRoot('rev-parse', 'HEAD');
// Chỉ mã của lab quyết định ứng viên; sửa tài liệu không làm ứng viên mơ hồ. reports/ đã bị ignore.
const dirty = gitRoot('status', '--porcelain', '--', 'examples/toi-noi-testing').length > 0;

// 1. Chạy lại các suite Vitest.
const vitestDirs = REGISTRY.filter((r) => r.source === 'vitest').map((r) => `articles/${r.labDir}`);
mkdirSync(new URL('../../reports/', import.meta.url), { recursive: true });
const jsonOut = 'reports/sys17-vitest.json';
const run = spawnSync(process.execPath, ['node_modules/vitest/vitest.mjs', 'run', ...vitestDirs, '--reporter=json', `--outputFile=${jsonOut}`],
  { cwd: LAB_ROOT, encoding: 'utf8' });
const vitest = JSON.parse(readFileSync(new URL(`../../${jsonOut}`, import.meta.url), 'utf8')) as {
  testResults: { name: string; assertionResults: { status: string }[] }[];
};
const runAt = new Date().toISOString();

const evidence: EvidenceItem[] = [];
const counts: Record<string, string> = {};
for (const r of REGISTRY) {
  if (r.source === 'vitest') {
    const files = vitest.testResults.filter((f) => f.name.replace(/\\/g, '/').includes(`/articles/${r.labDir}/`));
    const statuses = files.flatMap((f) => f.assertionResults.map((a) => a.status));
    const passed = statuses.filter((s) => s === 'passed').length;
    const failed = statuses.filter((s) => s === 'failed').length;
    counts[r.id] = `${passed}/${statuses.length} pass`;
    evidence.push({
      id: r.id, title: r.title, kind: r.kind, critical: r.critical, baselineOnly: r.baselineOnly,
      result: failed > 0 ? 'fail' : passed === 0 ? 'not_run' : 'pass',
      commit: dirty ? null : head, recordedAt: runAt, environment: `${process.platform}, Node ${process.version}, chạy lại bởi build-dossier`,
      watchPaths: r.watchPaths, note: counts[r.id],
    });
  } else {
    const rep = r.report!;
    const added = gitRoot('log', '--diff-filter=A', '--format=%H', '-1', '--', rep.path);
    evidence.push({
      id: r.id, title: r.title, kind: r.kind, critical: r.critical, baselineOnly: r.baselineOnly,
      result: rep.result, commit: added || null, recordedAt: rep.recordedAt, environment: rep.environment,
      watchPaths: r.watchPaths, note: `${rep.note} (${rep.path})`,
    });
  }
}

// 3. Diff từ commit của từng bằng chứng tới ứng viên.
const changedSince: Record<string, string[]> = {};
for (const c of new Set(evidence.map((e) => e.commit).filter((c): c is string => c !== null && c !== head))) {
  changedSince[c] = gitRoot('diff', '--name-only', c, head).split('\n').filter(Boolean);
}
const candidate: Candidate = { commit: head, changedSince, dirty };
const result = assess(evidence, candidate);

// 4. Ghi dossier.
const short = (c: string | null) => (c ? c.slice(0, 7) : '—');
const statusOf = new Map(result.items.map((i) => [i.id, i]));
const md = [
  `# Release dossier — ứng viên ${short(head)}`,
  '',
  `Dựng lúc ${runAt} · ${process.platform} · Node ${process.version} · working tree lab ${dirty ? 'CÓ thay đổi chưa commit' : 'sạch'} · vitest exit ${run.status}`,
  '',
  `**Đề xuất của luật: ${result.decision}** — chưa phải quyết định release; GO cần người ký có tên và vai trò.`,
  '',
  '| ID | Bằng chứng | Loại | Critical | Trạng thái | Commit | Ghi lúc | Môi trường | Ghi chú |',
  '|---|---|---|---|---|---|---|---|---|',
  ...evidence.map((e) => `| ${e.id} | ${e.title} | ${e.kind} | ${e.critical ? 'có' : 'không'} | ${statusOf.get(e.id)!.status} | ${short(e.commit)} | ${e.recordedAt.slice(0, 10)} | ${e.environment} | ${e.note ?? ''} |`),
  '',
  '## Chặn đề xuất', '', ...(result.blockers.length ? result.blockers.map((b) => `- ${b}`) : ['- Không có']),
  '', '## Rủi ro cần người ký chấp nhận hoặc xử lý', '', ...(result.risks.length ? result.risks.map((r) => `- ${r}`) : ['- Không có']),
  '', '## Sign-off', '', '| Người ký | Vai trò | Quyết định | Rủi ro chấp nhận | Ngày |', '|---|---|---|---|---|', '|  |  |  |  |  |', '',
].join('\n');
writeFileSync(new URL('../../reports/release-dossier.md', import.meta.url), md);
writeFileSync(new URL('../../reports/release-dossier.json', import.meta.url),
  JSON.stringify({ candidate: { commit: head, dirty }, runAt, evidence, assessment: result }, null, 2));

console.log(`Ứng viên ${short(head)} · vitest exit ${run.status} · đề xuất: ${result.decision}`);
for (const i of result.items) console.log(`  ${i.id.padEnd(8)} ${i.status.padEnd(14)} ${counts[i.id] ?? ''}`);
for (const b of result.blockers) console.log(`  CHẶN: ${b}`);
for (const r of result.risks) console.log(`  RỦI RO: ${r}`);
console.log('Đã ghi reports/release-dossier.md và .json');
if (process.argv.includes('--gate') && !['READY_FOR_SIGN_OFF', 'GO'].includes(result.decision)) process.exit(1);
