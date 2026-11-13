/**
 * SYS-17 · Kiểm luật gom bằng chứng (rules.ts) và tính toàn vẹn của danh mục (registry.ts).
 * Không chạy lại các suite khác — việc đó do build-dossier.ts làm.
 */
import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { REGISTRY } from './registry';
import { assess, type Candidate, type EvidenceItem, type SignOff } from './rules';

const C1 = 'c1'.repeat(20);
const C0 = 'c0'.repeat(20);
const SRC = 'examples/toi-noi-testing/src/';

const item = (over: Partial<EvidenceItem> & Pick<EvidenceItem, 'id'>): EvidenceItem => ({
  title: over.id, kind: 'automated', critical: true, result: 'pass', commit: C1, recordedAt: '2026-11-13',
  environment: 'lab', watchPaths: [SRC], ...over,
});
const cand = (over: Partial<Candidate> = {}): Candidate => ({ commit: C1, changedSince: {}, dirty: false, ...over });
const signer = (over: Partial<SignOff> = {}): SignOff => ({ name: 'Trưởng nhóm vận hành (giả định)', role: 'Product owner', decision: 'GO', acceptedRisks: [], ...over });

describe('SYS-17 · luật đề xuất release', () => {
  it('RD-01 · mọi mục critical đạt trên ứng viên nhưng chưa ai ký ⇒ READY_FOR_SIGN_OFF, không bao giờ tự GO', () => {
    const r = assess([item({ id: 'SYS-01' }), item({ id: 'SYS-03' })], cand());
    expect(r.decision).toBe('READY_FOR_SIGN_OFF');
    expect(r.blockers).toEqual([]);
  });

  it('RD-02 · một mục critical fail ⇒ NO_GO, kể cả khi đã có người ký GO', () => {
    const r = assess([item({ id: 'SYS-01' }), item({ id: 'SYS-13', result: 'fail' })], cand(), signer());
    expect(r.decision).toBe('NO_GO');
    expect(r.blockers).toEqual(['SYS-13 có ca thất bại']);
  });

  it('RD-03 · bằng chứng cũ: mã trong watchPaths đổi sau khi đo ⇒ stale, chặn đề xuất', () => {
    const r = assess([item({ id: 'SYS-05', commit: C0 })], cand({ changedSince: { [C0]: [`${SRC}web/customer.js`] } }));
    expect(r.items[0]).toMatchObject({ status: 'stale' });
    expect(r.decision).toBe('NOT_READY');
  });

  it('RD-04 · bằng chứng cũ nhưng chỉ tài liệu đổi ⇒ vẫn còn hiệu lực', () => {
    const r = assess([item({ id: 'SYS-05', commit: C0 })], cand({ changedSince: { [C0]: ['docs/system-testing/05-test-report.md'] } }));
    expect(r.items[0]).toMatchObject({ status: 'fresh_pass' });
    expect(r.decision).toBe('READY_FOR_SIGN_OFF');
  });

  it('RD-05 · mục con người critical chưa chạy (UAT) ⇒ NOT_READY dù mọi test tự động xanh', () => {
    const r = assess([item({ id: 'SYS-01' }), item({ id: 'SYS-15', kind: 'human', result: 'not_run' })], cand(), signer());
    expect(r.decision).toBe('NOT_READY');
    expect(r.blockers).toEqual(['SYS-15 chưa chạy']);
  });

  it('RD-06 · baseline hiệu năng không bao giờ được tính là đạt; là rủi ro, GO chỉ khi người ký chấp nhận đích danh', () => {
    const evidence = [item({ id: 'SYS-01' }), item({ id: 'SYS-07', kind: 'measured', critical: false, baselineOnly: true })];
    const risk = 'SYS-07: baseline máy lab, không phải bằng chứng SLO';
    expect(assess(evidence, cand()).risks).toEqual([risk]);
    expect(assess(evidence, cand(), signer()).decision).toBe('READY_FOR_SIGN_OFF');
    expect(assess(evidence, cand(), signer({ acceptedRisks: [risk] })).decision).toBe('GO');
  });

  it('RD-07 · sign-off thiếu tên hoặc vai trò bị từ chối; ứng viên có thay đổi chưa commit là INVALID_CANDIDATE', () => {
    expect(() => assess([item({ id: 'SYS-01' })], cand(), signer({ name: ' ' }))).toThrow('tên và vai trò');
    expect(assess([item({ id: 'SYS-01' })], cand({ dirty: true }), signer()).decision).toBe('INVALID_CANDIDATE');
  });

  it('RD-08 · bằng chứng không gắn được commit hoặc thiếu diff ⇒ stale, không mặc định là còn mới', () => {
    const r = assess([item({ id: 'SYS-01', commit: null }), item({ id: 'SYS-03', commit: C0 })], cand());
    expect(r.items.map((i) => i.status)).toEqual(['stale', 'stale']);
  });
});

describe('SYS-17 · danh mục bằng chứng', () => {
  const repo = fileURLToPath(new URL('../../../../', import.meta.url));

  it('RD-09 · đủ 16 bài, không trùng ID, mọi lab/báo cáo được trỏ tới đều tồn tại', () => {
    const ids = REGISTRY.map((r) => r.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (let n = 1; n <= 16; n++) expect(ids).toContain(`SYS-${String(n).padStart(2, '0')}`);
    for (const r of REGISTRY) {
      expect(r.watchPaths.length, r.id).toBeGreaterThan(0);
      if (r.source === 'vitest') expect(existsSync(`${repo}examples/toi-noi-testing/articles/${r.labDir}`), r.id).toBe(true);
      else expect(existsSync(`${repo}${r.report!.path}`), r.id).toBe(true);
    }
  });
});
