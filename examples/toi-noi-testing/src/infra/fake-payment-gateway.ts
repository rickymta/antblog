import {
  PaymentUnavailableError,
  type ChargeRequest,
  type ChargeResult,
  type PaymentGateway,
  type RefundRequest,
  type RefundResult,
} from '../app/ports';

/**
 * Chế độ lỗi của ví giả «VíXanh» (dùng cho test độ tin cậy):
 * - `none`          : chạy bình thường
 * - `decline`       : từ chối giao dịch (không nên thử lại)
 * - `error`         : ném PaymentUnavailableError ngay (lỗi tạm thời)
 * - `timeout`       : không bao giờ trả lời, KHÔNG trừ tiền
 * - `lost_response` : đã trừ tiền nhưng không bao giờ trả lời (mất phản hồi)
 */
export type FakeFailureMode = 'none' | 'decline' | 'error' | 'timeout' | 'lost_response';

interface ChargeRecord {
  chargeId: string;
  amount: number;
  refunded: boolean;
}

export class FakePaymentGateway implements PaymentGateway {
  /** Mọi lần gọi `charge` (kể cả lần thử lại) — để kiểm số lần gọi và khoá idempotency. */
  readonly calls: ChargeRequest[] = [];
  /** Độ trễ giả lập mỗi lần gọi (ms) — cho test hiệu năng. */
  latencyMs = 0;

  private mode: FakeFailureMode = 'none';
  private queue: FakeFailureMode[] = [];
  private readonly charges = new Map<string, ChargeRecord>();
  private readonly refunds = new Set<string>();
  private nextChargeNo = 1;

  /** Đặt chế độ lỗi lâu dài. */
  setMode(mode: FakeFailureMode): void {
    this.mode = mode;
  }

  /** Lỗi cho N lần gọi kế tiếp, sau đó quay về chế độ lâu dài: `failNext('timeout', 'timeout')`. */
  failNext(...modes: FakeFailureMode[]): void {
    this.queue.push(...modes);
  }

  /** Tổng tiền đã thực sự trừ (mỗi khoá idempotency tính một lần). */
  totalCharged(): number {
    let sum = 0;
    for (const record of this.charges.values()) if (!record.refunded) sum += record.amount;
    return sum;
  }

  chargedKeys(): string[] {
    return [...this.charges.keys()];
  }

  reset(): void {
    this.calls.length = 0;
    this.queue = [];
    this.mode = 'none';
    this.latencyMs = 0;
    this.charges.clear();
    this.refunds.clear();
  }

  async charge(request: ChargeRequest): Promise<ChargeResult> {
    this.calls.push({ ...request });
    const failure = this.queue.shift() ?? this.mode;
    if (this.latencyMs > 0) await sleep(this.latencyMs);

    if (failure === 'decline') return { status: 'declined', chargeId: null };
    if (failure === 'error') throw new PaymentUnavailableError();
    if (failure === 'timeout') return never();

    const record = this.applyCharge(request); // idempotent theo khoá
    if (failure === 'lost_response') return never();
    return { status: 'succeeded', chargeId: record.chargeId };
  }

  async refund(request: RefundRequest): Promise<RefundResult> {
    if (this.refunds.has(request.idempotencyKey)) return { status: 'succeeded' };
    const record = [...this.charges.values()].find((c) => c.chargeId === request.chargeId);
    if (!record || record.refunded || request.amount !== record.amount) return { status: 'rejected' };
    record.refunded = true;
    this.refunds.add(request.idempotencyKey);
    return { status: 'succeeded' };
  }

  private applyCharge(request: ChargeRequest): ChargeRecord {
    const existing = this.charges.get(request.idempotencyKey);
    if (existing) return existing;
    const record = { chargeId: `vx_${this.nextChargeNo++}`, amount: request.amount, refunded: false };
    this.charges.set(request.idempotencyKey, record);
    return record;
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Promise không bao giờ xong — mô phỏng cổng không trả lời. */
function never<T>(): Promise<T> {
  return new Promise<T>(() => {});
}
