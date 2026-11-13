/**
 * SYS-17 · Luật gom bằng chứng thành đề xuất release. Hàm thuần: không đọc file, không gọi git — dễ kiểm thử.
 * Chính sách (mục nào critical, ai được ký) là GIẢ ĐỊNH của bài tập, không phải quy định của Tới Nơi.
 */

export type EvidenceKind = 'automated' | 'measured' | 'drill' | 'human';
export type RawResult = 'pass' | 'fail' | 'not_run' | 'blocked';

export interface EvidenceItem {
  id: string; // SYS-01…SYS-16
  title: string;
  kind: EvidenceKind;
  /** Hỏng hoặc thiếu mục này thì không được đề xuất release. */
  critical: boolean;
  /** Chỉ là baseline trên máy lab — không chứng minh SLO của môi trường thật; không bao giờ tính là "đạt SLO". */
  baselineOnly?: boolean;
  result: RawResult;
  /** Commit mà bằng chứng được tạo ra trên đó (null nếu không xác định được). */
  commit: string | null;
  recordedAt: string; // ISO date
  environment: string;
  /** Đường dẫn mà thay đổi ở đó làm bằng chứng hết hiệu lực (tiền tố, so khớp đầu chuỗi). */
  watchPaths: string[];
  note?: string;
}

export interface Candidate {
  commit: string;
  /** File thay đổi giữa commit của từng bằng chứng và ứng viên: changedSince[evidenceCommit] = danh sách đường dẫn. */
  changedSince: Record<string, string[]>;
  /** Working tree có thay đổi chưa commit — ứng viên không xác định được nội dung. */
  dirty: boolean;
}

export type ItemStatus = 'fresh_pass' | 'fail' | 'missing' | 'stale' | 'baseline_only';

export interface SignOff {
  name: string;
  role: string;
  decision: 'GO' | 'NO_GO';
  acceptedRisks: string[];
}

export type Decision = 'NO_GO' | 'NOT_READY' | 'READY_FOR_SIGN_OFF' | 'GO' | 'INVALID_CANDIDATE';

export interface Assessment {
  decision: Decision;
  items: { id: string; status: ItemStatus; reason: string }[];
  blockers: string[];
  risks: string[];
}

export function itemStatus(e: EvidenceItem, c: Candidate): { status: ItemStatus; reason: string } {
  if (e.result === 'fail') return { status: 'fail', reason: `${e.id} có ca thất bại` };
  if (e.result === 'not_run' || e.result === 'blocked') return { status: 'missing', reason: `${e.id} ${e.result === 'blocked' ? 'bị chặn' : 'chưa chạy'}` };
  if (e.commit === null) return { status: 'stale', reason: `${e.id} không gắn được commit` };
  if (e.commit !== c.commit) {
    const changed = c.changedSince[e.commit];
    if (changed === undefined) return { status: 'stale', reason: `${e.id}: không có diff từ ${e.commit} tới ứng viên` };
    const hit = changed.find((f) => e.watchPaths.some((w) => f.startsWith(w)));
    if (hit) return { status: 'stale', reason: `${e.id}: ${hit} đổi sau khi đo` };
  }
  if (e.baselineOnly) return { status: 'baseline_only', reason: `${e.id}: baseline máy lab, không phải bằng chứng SLO` };
  return { status: 'fresh_pass', reason: `${e.id} đạt trên mã của ứng viên` };
}

export function assess(evidence: EvidenceItem[], candidate: Candidate, signOff?: SignOff): Assessment {
  const ids = new Set<string>();
  for (const e of evidence) {
    if (ids.has(e.id)) throw new Error(`Trùng mục bằng chứng ${e.id}`);
    ids.add(e.id);
  }
  const items = evidence.map((e) => ({ id: e.id, ...itemStatus(e, candidate) }));
  const byId = new Map(evidence.map((e) => [e.id, e]));
  const blockers = items.filter((i) => byId.get(i.id)!.critical && i.status !== 'fresh_pass').map((i) => i.reason);
  const risks = items.filter((i) => !byId.get(i.id)!.critical && i.status !== 'fresh_pass').map((i) => i.reason);

  if (candidate.dirty) return { decision: 'INVALID_CANDIDATE', items, blockers: ['Working tree có thay đổi chưa commit', ...blockers], risks };
  const criticalFail = items.some((i) => byId.get(i.id)!.critical && i.status === 'fail');
  if (criticalFail) return { decision: 'NO_GO', items, blockers, risks };
  if (blockers.length) return { decision: 'NOT_READY', items, blockers, risks };
  if (!signOff) return { decision: 'READY_FOR_SIGN_OFF', items, blockers, risks };

  if (!signOff.name.trim() || !signOff.role.trim()) throw new Error('Sign-off phải có tên và vai trò người chịu trách nhiệm');
  const unaccepted = risks.filter((r) => !signOff.acceptedRisks.includes(r));
  if (signOff.decision === 'GO' && unaccepted.length) {
    return { decision: 'READY_FOR_SIGN_OFF', items, blockers: unaccepted.map((r) => `Rủi ro chưa được chấp nhận: ${r}`), risks };
  }
  return { decision: signOff.decision, items, blockers, risks };
}
