/**
 * SYS-17 · Danh mục bằng chứng của series. Mục `vitest` được script chạy lại trên ứng viên;
 * các mục khác lấy từ báo cáo đã ghi (docs/system-testing/*) và gắn với commit đã thêm báo cáo đó.
 * critical / baselineOnly là chính sách GIẢ ĐỊNH của bài tập.
 */
import type { EvidenceKind, RawResult } from './rules';

const LAB = 'examples/toi-noi-testing/';
const CODE = [`${LAB}src/`, `${LAB}package-lock.json`, `${LAB}package.json`];

export interface RegistryEntry {
  id: string;
  title: string;
  kind: EvidenceKind;
  critical: boolean;
  baselineOnly?: boolean;
  /** Thư mục lab trong articles/ — mục vitest được chạy lại từ đây. */
  labDir?: string;
  source: 'vitest' | 'report';
  /** Với source=report: tệp báo cáo, kết quả đã ghi, môi trường và ghi chú. */
  report?: { path: string; result: RawResult; environment: string; recordedAt: string; note: string };
  watchPaths: string[];
}

const lab = (dir: string) => [...CODE, `${LAB}articles/${dir}/`];
const doc = (n: string) => `docs/system-testing/${n}`;

export const REGISTRY: RegistryEntry[] = [
  { id: 'SYS-01', title: 'Smoke', kind: 'automated', critical: true, source: 'vitest', labDir: 'system-testing-smoke-test', watchPaths: lab('system-testing-smoke-test') },
  { id: 'SYS-02', title: 'Sanity (thay đổi voucher)', kind: 'automated', critical: false, source: 'vitest', labDir: 'system-testing-sanity-test', watchPaths: lab('system-testing-sanity-test') },
  { id: 'SYS-03', title: 'Functional', kind: 'automated', critical: true, source: 'vitest', labDir: 'system-testing-functional-test', watchPaths: lab('system-testing-functional-test') },
  { id: 'SYS-04', title: 'Integration', kind: 'automated', critical: true, source: 'vitest', labDir: 'system-testing-integration-test', watchPaths: lab('system-testing-integration-test') },
  {
    id: 'SYS-05', title: 'System (Playwright)', kind: 'automated', critical: true, source: 'report',
    report: { path: doc('05-test-report.md'), result: 'pass', environment: 'Windows, Node 24.15.0, Playwright, Chrome', recordedAt: '2026-11-13', note: '3/3 hành trình khách/tài xế' },
    watchPaths: [...lab('system-testing-system-test'), `${LAB}src/web/`],
  },
  { id: 'SYS-06', title: 'Regression', kind: 'automated', critical: true, source: 'vitest', labDir: 'system-testing-regression-test', watchPaths: lab('system-testing-regression-test') },
  {
    id: 'SYS-07', title: 'Performance baseline', kind: 'measured', critical: false, baselineOnly: true, source: 'report',
    report: { path: doc('07-test-report.md'), result: 'pass', environment: 'i7-10700K, Node 24.15.0, memory store, 1 tiến trình', recordedAt: '2026-11-13', note: 'gate lab đạt; R-NFR-03 chưa đánh giá' },
    watchPaths: lab('system-testing-performance-test'),
  },
  {
    id: 'SYS-08', title: 'Load', kind: 'measured', critical: false, baselineOnly: true, source: 'report',
    report: { path: doc('08-test-report.md'), result: 'pass', environment: 'i7-10700K, Node 24.15.0, PGlite, 1 tiến trình', recordedAt: '2026-11-13', note: 'arrival-rate tới 30/s trên máy lab' },
    watchPaths: lab('system-testing-load-test'),
  },
  {
    id: 'SYS-09', title: 'Stress', kind: 'measured', critical: false, baselineOnly: true, source: 'report',
    report: { path: doc('09-test-report.md'), result: 'pass', environment: 'i7-10700K, Node 24.15.0, PGlite, 1 tiến trình', recordedAt: '2026-11-13', note: 'không chạm điểm gãy; hồi phục sau tải cao' },
    watchPaths: lab('system-testing-stress-test'),
  },
  { id: 'SYS-10', title: 'Security (lab scope)', kind: 'automated', critical: true, source: 'vitest', labDir: 'system-testing-security-pentest', watchPaths: lab('system-testing-security-pentest') },
  {
    id: 'SYS-11', title: 'Compatibility (Playwright)', kind: 'automated', critical: true, source: 'report',
    report: { path: doc('11-test-report.md'), result: 'pass', environment: 'Windows, Chrome + Edge, viewport matrix', recordedAt: '2026-11-13', note: 'emulation không thay thiết bị thật' },
    watchPaths: [...lab('system-testing-compatibility-test'), `${LAB}src/web/`],
  },
  {
    id: 'SYS-12', title: 'Usability study', kind: 'human', critical: false, source: 'report',
    report: { path: doc('12-test-report.md'), result: 'not_run', environment: '—', recordedAt: '2026-11-13', note: 'chưa có participant; chỉ có kiểm UI bổ trợ' },
    watchPaths: [`${LAB}src/web/`],
  },
  { id: 'SYS-13', title: 'Data integrity', kind: 'automated', critical: true, source: 'vitest', labDir: 'system-testing-data-integrity-test', watchPaths: lab('system-testing-data-integrity-test') },
  { id: 'SYS-14', title: 'Recovery (ví lỗi)', kind: 'automated', critical: true, source: 'vitest', labDir: 'system-testing-recovery-test', watchPaths: lab('system-testing-recovery-test') },
  {
    id: 'SYS-14R', title: 'Restore drill PostgreSQL', kind: 'drill', critical: false, source: 'report',
    report: { path: doc('14-test-report.md'), result: 'pass', environment: 'PostgreSQL 18.3 cluster tạm', recordedAt: '2026-11-13', note: 'marker tổng hợp, không restore schema Tới Nơi' },
    watchPaths: [`${LAB}articles/system-testing-recovery-test/restore-drill.ps1`],
  },
  {
    id: 'SYS-15', title: 'UAT sign-off', kind: 'human', critical: true, source: 'report',
    report: { path: doc('15-test-report.md'), result: 'not_run', environment: '—', recordedAt: '2026-11-13', note: 'supporting checks pass; chưa có phiên UAT và người ký' },
    watchPaths: CODE,
  },
  { id: 'SYS-16', title: 'Deployment', kind: 'automated', critical: true, source: 'vitest', labDir: 'system-testing-deployment-test', watchPaths: [...lab('system-testing-deployment-test'), `${LAB}src/server.ts`] },
];
