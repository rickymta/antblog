/**
 * STATIC ANALYSIS tự viết, tối giản — chỉ để minh hoạ cho bài TEST-022.
 * Đọc mã nguồn dưới dạng VĂN BẢN, không import, không gọi hàm nào của tệp được kiểm.
 * Công cụ thật (tsc --strict, ESLint) ở bài TEST-029.
 *
 * Hai quy tắc:
 *   S1 — không viết cứng hệ số cao điểm 1.25; dùng PEAK_PERCENT (SPEC R-FARE-04).
 *   S2 — dòng nhân hệ số cao điểm không được chứa phụ phí mưa (SPEC R-FARE-03, R-FARE-05).
 */
export interface Finding {
  rule: 'S1' | 'S2';
  line: number;
  text: string;
  message: string;
}

/** Bỏ chú thích để quy tắc chỉ nhìn vào mã. Đủ dùng cho tệp mẫu, không phải trình phân tích cú pháp thật. */
function codeOnly(line: string): string {
  return line.replace(/\/\/.*$/, '').replace(/\/\*.*?\*\//g, '').replace(/^\s*\*.*$/, '');
}

export function analyzeFareSource(source: string): Finding[] {
  const findings: Finding[] = [];
  source.split(/\r?\n/).forEach((raw, index) => {
    const code = codeOnly(raw);
    const isPeakLine = /\b1\.25\b/.test(code) || (/\bPEAK_PERCENT\b/.test(code) && /percentOf|\*/.test(code));
    if (/\b1\.25\b/.test(code)) {
      findings.push({ rule: 'S1', line: index + 1, text: raw.trim(), message: 'Hệ số cao điểm viết cứng 1.25 — dùng PEAK_PERCENT.' });
    }
    if (isPeakLine && /rain/i.test(code)) {
      findings.push({ rule: 'S2', line: index + 1, text: raw.trim(), message: 'Phụ phí mưa nằm trong phép nhân hệ số cao điểm.' });
    }
  });
  return findings;
}
