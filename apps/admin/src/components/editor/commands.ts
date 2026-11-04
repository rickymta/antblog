// Lệnh định dạng Markdown thao tác trực tiếp trên EditorView của CodeMirror 6 (một transaction = một bước hoàn tác).
import { EditorSelection, EditorView } from '@uiw/react-codemirror';

/**
 * Bọc vùng chọn bằng `before`/`after` (đậm, nghiêng, mã nội dòng…). Vùng chọn đã được bọc đúng như vậy ⇒ gỡ bọc.
 * Không chọn gì ⇒ chèn `before + placeholder + after` và chọn sẵn chữ mẫu để gõ đè.
 */
export function toggleWrap(view: EditorView, before: string, after: string, placeholder: string): void {
  const { state } = view;
  const tr = state.changeByRange((range) => {
    const text = state.sliceDoc(range.from, range.to);
    const outerFrom = range.from - before.length;
    const outerTo = range.to + after.length;
    // Đã bọc bên ngoài vùng chọn ⇒ gỡ.
    if (outerFrom >= 0 && outerTo <= state.doc.length && state.sliceDoc(outerFrom, range.from) === before && state.sliceDoc(range.to, outerTo) === after) {
      return {
        changes: [
          { from: outerFrom, to: range.from, insert: '' },
          { from: range.to, to: outerTo, insert: '' },
        ],
        range: EditorSelection.range(outerFrom, range.to - before.length),
      };
    }
    // Vùng chọn gồm cả dấu bọc ⇒ gỡ.
    if (text.length >= before.length + after.length && text.startsWith(before) && text.endsWith(after) && text.length > 0) {
      const inner = text.slice(before.length, text.length - after.length);
      return { changes: { from: range.from, to: range.to, insert: inner }, range: EditorSelection.range(range.from, range.from + inner.length) };
    }
    const body = text || placeholder;
    return {
      changes: { from: range.from, to: range.to, insert: before + body + after },
      range: EditorSelection.range(range.from + before.length, range.from + before.length + body.length),
    };
  });
  view.dispatch(state.update(tr, { scrollIntoView: true, userEvent: 'input' }));
  view.focus();
}

/** Các dòng chạm vào vùng chọn (không trùng). */
function selectedLines(view: EditorView) {
  const { state } = view;
  const seen = new Set<number>();
  const lines = [];
  for (const r of state.selection.ranges) {
    for (let n = state.doc.lineAt(r.from).number; n <= state.doc.lineAt(r.to).number; n++) {
      if (seen.has(n)) continue;
      seen.add(n);
      lines.push(state.doc.line(n));
    }
  }
  return lines.sort((a, b) => a.number - b.number);
}

/** Tiêu đề `## ` / `### `: đặt cấp cho các dòng đang chọn; đã đúng cấp ⇒ bỏ tiêu đề. */
export function toggleHeading(view: EditorView, level: 2 | 3): void {
  const prefix = `${'#'.repeat(level)} `;
  const lines = selectedLines(view);
  const allSame = lines.every((l) => l.text.startsWith(prefix) && !l.text.startsWith(prefix + '#'));
  const changes = lines.map((l) => {
    const existing = /^#{1,6}\s+/.exec(l.text)?.[0] ?? '';
    return { from: l.from, to: l.from + existing.length, insert: allSame ? '' : prefix };
  });
  view.dispatch({ changes, scrollIntoView: true, userEvent: 'input' });
  view.focus();
}

/** Tiền tố dòng (`> `, `- `, `1. `): bật/tắt cho mọi dòng đang chọn. `numbered` ⇒ đánh số 1., 2., … */
export function toggleLinePrefix(view: EditorView, kind: 'quote' | 'bullet' | 'numbered'): void {
  const lines = selectedLines(view);
  const re = kind === 'quote' ? /^>\s?/ : kind === 'bullet' ? /^[-*+]\s+/ : /^\d+[.)]\s+/;
  const allHave = lines.filter((l) => l.text.trim()).every((l) => re.test(l.text));
  let n = 0;
  const changes = lines
    .filter((l) => allHave || l.text.trim() || lines.length === 1)
    .map((l) => {
      const existing = re.exec(l.text)?.[0] ?? '';
      if (allHave) return { from: l.from, to: l.from + existing.length, insert: '' };
      n++;
      const prefix = kind === 'quote' ? '> ' : kind === 'bullet' ? '- ' : `${n}. `;
      // Đổi kiểu danh sách: bỏ tiền tố kiểu khác trước.
      const other = /^([-*+]|\d+[.)])\s+/.exec(l.text)?.[0] ?? '';
      return { from: l.from, to: l.from + (existing || (kind !== 'quote' ? other : '')).length, insert: prefix };
    });
  view.dispatch({ changes, scrollIntoView: true, userEvent: 'input' });
  view.focus();
}

/**
 * Chèn một khối (bảng, khối mã, ghi chú…) thành đoạn riêng: tự thêm dòng trống trước/sau khi cần.
 * `select` = [from, to] tương đối trong `block` để chọn sẵn (vd chữ mẫu); vắng ⇒ đặt con trỏ cuối khối.
 */
export function insertBlock(view: EditorView, block: string, select?: [number, number]): void {
  const { state } = view;
  const range = state.selection.main;
  const lineStart = state.doc.lineAt(range.from);
  const lineEnd = state.doc.lineAt(range.to);
  const before = state.sliceDoc(0, range.from);
  const after = state.sliceDoc(range.to);
  // Đang đứng giữa dòng có chữ ⇒ xuống dòng; cần một dòng trống ngăn cách với đoạn phía trên/dưới.
  let lead = '';
  if (before.length) {
    if (range.from > lineStart.from) lead = '\n\n';
    else if (!before.endsWith('\n\n')) lead = '\n';
  }
  let trail = '';
  if (after.length) {
    if (range.to < lineEnd.to) trail = '\n\n';
    else if (!after.startsWith('\n\n')) trail = '\n';
  } else trail = '\n';
  const insert = lead + block + trail;
  const base = range.from + lead.length;
  const sel = select ? EditorSelection.range(base + select[0], base + select[1]) : EditorSelection.cursor(base + block.length);
  view.dispatch({ changes: { from: range.from, to: range.to, insert }, selection: sel, scrollIntoView: true, userEvent: 'input' });
  view.focus();
}

/** Khối mã có ngôn ngữ: vùng chọn (nếu có) thành nội dung khối. */
export function insertCodeBlock(view: EditorView, lang: string, placeholder: string): void {
  const { state } = view;
  const range = state.selection.main;
  const selected = state.sliceDoc(range.from, range.to);
  const body = selected || placeholder;
  const open = '```' + lang + '\n';
  insertBlock(view, `${open}${body}\n\`\`\``, [open.length, open.length + body.length]);
}

/** `[chữ](url)`: có vùng chọn ⇒ thành chữ của liên kết, chọn sẵn `url`; vùng chọn là URL ⇒ thành địa chỉ. */
export function insertLink(view: EditorView, placeholder: string): void {
  const { state } = view;
  const tr = state.changeByRange((range) => {
    const text = state.sliceDoc(range.from, range.to);
    if (/^https?:\/\/\S+$/i.test(text)) {
      const insert = `[${placeholder}](${text})`;
      return { changes: { from: range.from, to: range.to, insert }, range: EditorSelection.range(range.from + 1, range.from + 1 + placeholder.length) };
    }
    const label = text || placeholder;
    const insert = `[${label}](https://)`;
    const urlFrom = range.from + label.length + 3;
    return { changes: { from: range.from, to: range.to, insert }, range: EditorSelection.range(urlFrom, urlFrom + 'https://'.length) };
  });
  view.dispatch(state.update(tr, { scrollIntoView: true, userEvent: 'input' }));
  view.focus();
}

/** Chèn chữ tại con trỏ (thay vùng chọn), đặt con trỏ sau chữ chèn. */
export function insertText(view: EditorView, text: string): void {
  const range = view.state.selection.main;
  view.dispatch({
    changes: { from: range.from, to: range.to, insert: text },
    selection: EditorSelection.cursor(range.from + text.length),
    scrollIntoView: true,
    userEvent: 'input',
  });
  view.focus();
}

/** Thay lần xuất hiện đầu tiên của `marker` trong tài liệu (chỗ giữ chỗ khi tải ảnh). Trả false nếu không còn. */
export function replaceMarker(view: EditorView, marker: string, insert: string): boolean {
  const idx = view.state.doc.toString().indexOf(marker);
  if (idx < 0) return false;
  view.dispatch({ changes: { from: idx, to: idx + marker.length, insert }, userEvent: 'input' });
  return true;
}

/** Nhảy tới dòng `line` (1-based): đặt con trỏ đầu dòng, cuộn lên gần đầu khung. */
export function goToLine(view: EditorView, line: number): void {
  const l = view.state.doc.line(Math.min(Math.max(1, line), view.state.doc.lines));
  view.dispatch({ selection: EditorSelection.cursor(l.from), effects: EditorView.scrollIntoView(l.from, { y: 'start', yMargin: 24 }) });
  view.focus();
}

export interface OutlineItem {
  line: number;
  text: string;
  depth: 2 | 3;
}

/** Tiêu đề H2/H3 kèm số dòng — cùng luật bỏ khối mã với `extractHeadings` của @tkh/ui (để ghép theo thứ tự). */
export function outlineLines(markdown: string): OutlineItem[] {
  const out: OutlineItem[] = [];
  let inFence = false;
  markdown.split(/\r?\n/).forEach((raw, i) => {
    const line = raw.trimEnd();
    if (/^(```|~~~)/.test(line.trimStart())) {
      inFence = !inFence;
      return;
    }
    if (inFence) return;
    const m = /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(line);
    if (!m) return;
    const depth = m[1]!.length;
    if (depth === 2 || depth === 3) out.push({ line: i + 1, text: m[2]!, depth });
  });
  return out;
}
