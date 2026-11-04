import { PaymentUnavailableError, type ChargeRequest, type PaymentGateway } from './ports';

export class PaymentTimeoutError extends Error {
  constructor(timeoutMs: number) {
    super(`Cổng thanh toán không trả lời sau ${timeoutMs} ms`);
    this.name = 'PaymentTimeoutError';
  }
}

export interface PaymentOutcome {
  status: 'paid' | 'failed';
  attempts: number;
  failure: 'declined' | 'timeout' | 'unavailable' | null;
  chargeId: string | null;
}

/** Chờ một promise tối đa `timeoutMs`; quá hạn ⇒ PaymentTimeoutError. */
export async function withTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new PaymentTimeoutError(timeoutMs)), timeoutMs);
  });
  try {
    return await Promise.race([promise, timeout]);
  } finally {
    clearTimeout(timer);
  }
}

/**
 * Thu tiền với hết giờ + thử lại (SPEC R-PAY-04):
 * mỗi lần chờ tối đa `timeoutMs`; hết giờ hoặc lỗi tạm thời ⇒ thử lại tới `maxRetries` lần
 * với CÙNG khoá idempotency; bị từ chối ⇒ dừng ngay.
 */
export async function chargeWithRetry(
  gateway: PaymentGateway,
  request: ChargeRequest,
  options: { timeoutMs: number; maxRetries: number },
): Promise<PaymentOutcome> {
  let lastFailure: PaymentOutcome['failure'] = null;
  const maxAttempts = 1 + options.maxRetries;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      const result = await withTimeout(gateway.charge(request), options.timeoutMs);
      if (result.status === 'succeeded') {
        return { status: 'paid', attempts: attempt, failure: null, chargeId: result.chargeId };
      }
      return { status: 'failed', attempts: attempt, failure: 'declined', chargeId: null };
    } catch (error) {
      if (error instanceof PaymentTimeoutError) lastFailure = 'timeout';
      else if (error instanceof PaymentUnavailableError) lastFailure = 'unavailable';
      else throw error; // lỗi lập trình — không nuốt
    }
  }
  return { status: 'failed', attempts: maxAttempts, failure: lastFailure, chargeId: null };
}
