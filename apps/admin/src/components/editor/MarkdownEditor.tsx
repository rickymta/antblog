// Trình soạn Markdown CodeMirror 6 + thanh công cụ. Dán/kéo thả ảnh ⇒ tải lên thư viện media, chỗ giữ chỗ
// "Đang tải ảnh…" được thay bằng `![alt](url)` khi xong.
import { useEffect, useMemo, useRef, useState, type KeyboardEvent as ReactKeyboardEvent, type ReactNode } from 'react';
import CodeMirror, { EditorView, Prec, keymap, type Extension } from '@uiw/react-codemirror';
import { markdown, markdownLanguage } from '@codemirror/lang-markdown';
import { languages } from '@codemirror/language-data';
// @codemirror/commands là phụ thuộc của @uiw/react-codemirror (basicSetup) — cùng bản với lịch sử hoàn tác đang chạy.
import { openSearchPanel } from '@codemirror/search';
import { redo, undo } from '@codemirror/commands';
import { useTranslation } from 'react-i18next';
import {
  ChevronDown,
  Code,
  Heading,
  SlidersHorizontal,
  Search,
  ListChecks,
  Type,
  ImagePlus,
  Italic,
  Link as LinkIcon,
  List,
  ListOrdered,
  MessageSquareWarning,
  Minus,
  Redo2,
  SquareCode,
  Table,
  TextQuote,
  Undo2,
  CircleQuestionMark,
  type LucideIcon,
} from 'lucide-react';
import { api } from '@tkh/shared';
import { cx } from '../ui';
import { errorMessage } from '../../lib/errors';
import { checkImageFile, defaultAlt, imageMarkdown } from '../../lib/media';
import { useToast } from '../../lib/toast';
import { insertBlock, insertCodeBlock, insertLink, replaceMarker, toggleHeading, toggleLinePrefix, toggleWrap } from './commands';

import { liveMarkdown } from './liveMarkdown';

export const CODE_LANGUAGES = ['sql', 'bash', 'javascript', 'typescript', 'json', 'yaml', 'python', 'go', 'java', 'csharp', 'html', 'css', 'dockerfile', 'text'] as const;

/** Khoá màu theo theme trang (class `.dark` trên <html>) — đổi tức thì khi bấm nút sáng/tối. */
export function useDarkClass(): boolean {
  const [dark, setDark] = useState(() => document.documentElement.classList.contains('dark'));
  useEffect(() => {
    const obs = new MutationObserver(() => setDark(document.documentElement.classList.contains('dark')));
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    return () => obs.disconnect();
  }, []);
  return dark;
}

const editorTheme = EditorView.theme({
  '&': { height: '100%', fontSize: '14px', backgroundColor: 'transparent' },
  '&.cm-focused': { outline: 'none' },
  '.cm-scroller': { fontFamily: '"JetBrains Mono", ui-monospace, "Cascadia Mono", Consolas, monospace', lineHeight: '1.7' },
  '.cm-content': { padding: '16px 0 40vh', caretColor: 'var(--color-primary)' },
  '.cm-line': { padding: '0 18px' },
  '.cm-gutters': { display: 'none' },
  '.cm-placeholder': { color: 'var(--tkh-muted)' },
});

type ImageUploader = (view: EditorView, files: File[], pos?: number) => void;

const toolbarGroups = ['format', 'blocks', 'insert', 'editing'] as const;
type ToolbarGroup = typeof toolbarGroups[number];
const toolbarKey = 'cms.article.toolbar.v1';

function readToolbar(): ToolbarGroup[] {
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(toolbarKey) ?? 'null');
    if (Array.isArray(saved)) return toolbarGroups.filter((group) => saved.includes(group));
  } catch {
    // Browser storage may be unavailable.
  }
  return [...toolbarGroups];
}

/** Thanh công cụ định dạng — nhận view của CodeMirror (null khi chưa dựng xong). */
export function MarkdownToolbar({ view, onImage, extra }: { view: EditorView | null; onImage: () => void; extra?: ReactNode }) {
  const { t } = useTranslation();
  const run = (fn: (v: EditorView) => void) => () => {
    if (view) fn(view);
  };
  const disabled = !view;
  const [groups, setGroups] = useState(readToolbar);
  const [settings, setSettings] = useState(false);
  function configure(next: ToolbarGroup[]) {
    setGroups(next);
    try {
      localStorage.setItem(toolbarKey, JSON.stringify(next));
    } catch {
      // Keep the preference for this session.
    }
  }

  return (
    <div className="min-w-0">
    <div role="toolbar" aria-label={t('cms.toolbar')} className="flex flex-wrap items-center gap-0.5">
      {groups.includes('format') && <>
      <ToolMenu icon={Heading} label={t('cms.headings')} menuLabel={t('cms.headings')} disabled={disabled}
        items={([1, 2, 3, 4, 5, 6] as const).map(level => ({
          key: String(level), label: `H${level}`, onSelect: run(v => toggleHeading(v, level)),
        }))} />
      <ToolButton glyph={<span className="font-bold">B</span>} label={t('cms.tbBold')} disabled={disabled} onClick={run((v) => toggleWrap(v, '**', '**', t('cms.snipBold')))} />
      <ToolButton icon={Italic} label={t('cms.tbItalic')} disabled={disabled} onClick={run((v) => toggleWrap(v, '*', '*', t('cms.snipItalic')))} />
      <ToolButton icon={Code} label={t('cms.tbCode')} disabled={disabled} onClick={run((v) => toggleWrap(v, '`', '`', t('cms.snipCode')))} />
      <ToolMenu icon={Type} label={t('cms.moreFormats')} menuLabel={t('cms.moreFormats')} disabled={disabled}
        items={([
          ['underline', '<u>', '</u>'], ['strike', '~~', '~~'], ['highlight', '<mark>', '</mark>'],
          ['superscript', '<sup>', '</sup>'], ['subscript', '<sub>', '</sub>'],
        ] as const).map(([key, start, end]) => ({
          key, label: t(`cms.${key}`), onSelect: run(v => toggleWrap(v, start, end, t('cms.sampleText'))),
        }))} />
      <ToolButton icon={LinkIcon} label={t('cms.tbLink')} disabled={disabled} onClick={run((v) => insertLink(v, t('cms.snipLink')))} />
      </>}
      {groups.includes('blocks') && <>
      <ToolMenu
        icon={SquareCode}
        label={t('cms.tbCodeBlock')}
        menuLabel={t('cms.codeLanguage')}
        disabled={disabled}
        items={CODE_LANGUAGES.map((lang) => ({
          key: lang,
          label: lang === 'text' ? t('cms.langPlain') : lang,
          mono: lang !== 'text',
          onSelect: run((v) => insertCodeBlock(v, lang, '')),
        }))}
      />
      <ToolButton icon={TextQuote} label={t('cms.tbQuote')} disabled={disabled} onClick={run((v) => toggleLinePrefix(v, 'quote'))} />
      <ToolButton icon={List} label={t('cms.tbBullet')} disabled={disabled} onClick={run((v) => toggleLinePrefix(v, 'bullet'))} />
      <ToolButton icon={ListOrdered} label={t('cms.tbNumbered')} disabled={disabled} onClick={run((v) => toggleLinePrefix(v, 'numbered'))} />
      <ToolButton icon={ListChecks} label={t('cms.checklist')} disabled={disabled} onClick={run(v => toggleLinePrefix(v, 'task'))} />
      <ToolButton
        icon={Table}
        label={t('cms.tbTable')}
        disabled={disabled}
        onClick={run((v) => {
          const c = t('cms.snipColumn');
          const cell = t('cms.snipCell');
          const block = `| ${c} 1 | ${c} 2 | ${c} 3 |\n| --- | --- | --- |\n| ${cell} | ${cell} | ${cell} |\n| ${cell} | ${cell} | ${cell} |`;
          insertBlock(v, block, [2, 2 + c.length + 2]);
        })}
      />
      <ToolButton icon={Minus} label={t('cms.tbHr')} disabled={disabled} onClick={run((v) => insertBlock(v, '---'))} />
      </>}
      {groups.includes('insert') && <>
      <ToolMenu
        icon={MessageSquareWarning}
        label={t('cms.tbCallout')}
        menuLabel={t('cms.tbCallout')}
        disabled={disabled}
        items={(
          [
            ['', t('cms.calloutInfo')],
            [' tkh-callout--warn', t('cms.calloutWarn')],
            [' tkh-callout--example', t('cms.calloutExample')],
          ] as const
        ).map(([variant, label]) => ({
          key: variant || 'info',
          label,
          onSelect: run((v) => {
            const title = t('cms.snipCalloutTitle');
            const open = `<div class="tkh-callout${variant}">\n<span class="tkh-callout-title">`;
            insertBlock(v, `${open}${title}</span>\n<p>${t('cms.snipCalloutBody')}</p>\n</div>`, [open.length, open.length + title.length]);
          }),
        }))}
      />
      <ToolButton
        icon={CircleQuestionMark}
        label={t('cms.tbQuiz')}
        disabled={disabled}
        onClick={run((v) => {
          const q = t('cms.snipQuizQuestion');
          const opt = t('cms.snipQuizOption');
          const open = '<div class="tkh-callout tkh-callout--example">\n<span class="tkh-callout-title">';
          const block =
            `${open}${q}</span>\n<p><b>A.</b> ${opt} A</p>\n<p><b>B.</b> ${opt} B</p>\n<p><b>C.</b> ${opt} C</p>\n` +
            `<details>\n<summary><b>${t('cms.snipQuizAnswer')}</b></summary>\n<p>${t('cms.snipQuizExplain')}</p>\n</details>\n</div>`;
          insertBlock(v, block, [open.length, open.length + q.length]);
        })}
      />
      <ToolButton icon={ImagePlus} label={t('cms.tbImage')} disabled={disabled} onClick={onImage} />
      </>}
      {groups.includes('editing') && <>
      <ToolButton icon={Undo2} label={t('cms.tbUndo')} disabled={disabled} onClick={run((v) => { undo(v); v.focus(); })} />
      <ToolButton icon={Redo2} label={t('cms.tbRedo')} disabled={disabled} onClick={run((v) => { redo(v); v.focus(); })} />
      <ToolButton icon={Search} label={t('cms.findReplace')} disabled={disabled} onClick={run(openSearchPanel)} />
      </>}
      <button type="button" aria-expanded={settings} title={t('cms.toolbarSettings')} aria-label={t('cms.toolbarSettings')}
        className="inline-flex size-8 items-center justify-center rounded-ui hover:bg-surface-2"
        onClick={() => setSettings(!settings)}><SlidersHorizontal className="size-4" /></button>
      {extra}
    </div>
    {settings && <fieldset className="my-2 flex flex-wrap items-center gap-3 rounded-ui border border-line px-3 py-2 text-sm">
      <legend className="px-1 font-semibold">{t('cms.toolbarSettings')}</legend>
      {toolbarGroups.map(group => <label key={group} className="flex items-center gap-1">
        <input type="checkbox" checked={groups.includes(group)}
          onChange={e => configure(e.target.checked ? [...groups, group] : groups.filter(item => item !== group))} />
        {t(`cms.group_${group}`)}
      </label>)}
      <button type="button" className="text-primary" onClick={() => configure(['format', 'editing'])}>{t('cms.basicTools')}</button>
      <button type="button" className="text-primary" onClick={() => configure([...toolbarGroups])}>{t('cms.allTools')}</button>
    </fieldset>}
    </div>
  );
}

function ToolButton({
  icon: Icon,
  glyph,
  label,
  onClick,
  disabled,
}: {
  icon?: LucideIcon;
  glyph?: ReactNode;
  label: string;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      disabled={disabled}
      // Giữ focus + vùng chọn trong trình soạn khi bấm nút.
      onMouseDown={(e) => e.preventDefault()}
      onClick={onClick}
      className="inline-flex size-8 items-center justify-center rounded-ui text-sm text-ink transition-colors hover:bg-surface-2 disabled:opacity-40"
    >
      {Icon ? <Icon className="size-4" aria-hidden /> : <span aria-hidden>{glyph}</span>}
    </button>
  );
}

/** Nút mở danh sách lựa chọn (ngôn ngữ khối mã, kiểu ghi chú). Esc/bấm ra ngoài để đóng, mũi tên để di chuyển. */
function ToolMenu({
  icon: Icon,
  label,
  menuLabel,
  items,
  disabled,
}: {
  icon: LucideIcon;
  label: string;
  menuLabel: string;
  items: { key: string; label: string; mono?: boolean; onSelect: () => void }[];
  disabled?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const wrap = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLUListElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (!wrap.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', onDown);
    menu.current?.querySelector<HTMLButtonElement>('button')?.focus();
    return () => document.removeEventListener('mousedown', onDown);
  }, [open]);

  const onMenuKey = (e: ReactKeyboardEvent) => {
    const buttons = Array.from(menu.current?.querySelectorAll<HTMLButtonElement>('button') ?? []);
    const idx = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      setOpen(false);
      trigger.current?.focus();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      buttons[(idx + 1) % buttons.length]?.focus();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      buttons[(idx - 1 + buttons.length) % buttons.length]?.focus();
    } else if (e.key === 'Tab') {
      setOpen(false);
    }
  };

  return (
    <div ref={wrap} className="relative">
      <button
        ref={trigger}
        type="button"
        aria-label={label}
        title={label}
        aria-haspopup="menu"
        aria-expanded={open}
        disabled={disabled}
        onClick={() => setOpen((o) => !o)}
        className="inline-flex h-8 items-center gap-0.5 rounded-ui px-1.5 text-ink transition-colors hover:bg-surface-2 disabled:opacity-40"
      >
        <Icon className="size-4" aria-hidden />
        <ChevronDown className="size-3 text-muted" aria-hidden />
      </button>
      {open && (
        <ul
          ref={menu}
          role="menu"
          aria-label={menuLabel}
          onKeyDown={onMenuKey}
          className="absolute top-full left-0 z-30 mt-1 max-h-72 min-w-[11rem] overflow-y-auto rounded-ui border border-line bg-surface p-1 shadow-lg"
        >
          {items.map((it) => (
            <li key={it.key} role="none">
              <button
                type="button"
                role="menuitem"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => {
                  setOpen(false);
                  it.onSelect();
                }}
                className={cx('block w-full rounded-[6px] px-2.5 py-1.5 text-left text-sm text-ink hover:bg-surface-2 focus:bg-surface-2 focus:outline-none', it.mono && 'font-mono')}
              >
                {it.label}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** Ô soạn CodeMirror. `onView` nhận EditorView khi dựng xong (để thanh công cụ, mục lục, cuộn đồng bộ dùng). */
export function MarkdownCodeMirror({
  live,
  value,
  onChange,
  onView,
  placeholder,
  label,
  invalid,
}: {
  live: boolean;
  value: string;
  onChange: (v: string) => void;
  onView: (view: EditorView | null) => void;
  placeholder: string;
  label: string;
  invalid?: boolean;
}) {
  const { t, i18n } = useTranslation();
  const toast = useToast();
  const dark = useDarkClass();

  // Hàm tải ảnh luôn đọc bản mới nhất của t/toast — extension chỉ dựng một lần.
  const uploaderRef = useRef<ImageUploader>(() => undefined);
  uploaderRef.current = (view, files, pos) => {
    for (const file of files) {
      const bad = checkImageFile(file);
      if (bad) {
        toast.error(t('cms.uploadFailed', { name: file.name || 'image', message: bad === 'type' ? t('media.errType') : t('media.errSize') }));
        continue;
      }
      const marker = `![${t('cms.uploadingImage')}](#tkh-uploading-${crypto.randomUUID().slice(0, 8)})`;
      const at = pos ?? view.state.selection.main.head;
      const line = view.state.doc.lineAt(at);
      // Ảnh thành đoạn riêng nếu đang ở giữa một dòng có chữ.
      const lead = at > line.from && line.text.trim() ? '\n\n' : '';
      view.dispatch({ changes: { from: at, to: at, insert: `${lead}${marker}\n` }, userEvent: 'input' });
      void api.admin.media
        .upload(file)
        .then((item) => {
          // Người dùng đã xoá chỗ giữ chỗ ⇒ không chèn lại (ảnh vẫn nằm trong thư viện).
          replaceMarker(view, marker, imageMarkdown(item.url, defaultAlt(item)));
        })
        .catch((err: unknown) => {
          if (!replaceMarker(view, `${marker}\n`, '')) replaceMarker(view, marker, '');
          toast.error(t('cms.uploadFailed', { name: file.name || 'image', message: errorMessage(err, t, i18n.language) }));
        });
    }
  };

  const extensions = useMemo<Extension[]>(
    () => [
      markdown({ base: markdownLanguage, codeLanguages: languages }),
      EditorView.lineWrapping,
      // Ưu tiên cao nhất để nền trong suốt + font đè lên theme sáng/tối có sẵn.
      ...(live ? [Prec.highest(liveMarkdown)] : []),
      Prec.highest(editorTheme),
      EditorView.contentAttributes.of({ 'aria-label': label, spellcheck: 'false' }),
      Prec.highest(
        keymap.of([
          { key: 'Mod-b', run: (v) => (toggleWrap(v, '**', '**', t('cms.snipBold')), true) },
          { key: 'Mod-i', run: (v) => (toggleWrap(v, '*', '*', t('cms.snipItalic')), true) },
          { key: 'Mod-k', run: (v) => (insertLink(v, t('cms.snipLink')), true) },
        ]),
      ),
      EditorView.domEventHandlers({
        paste(event, view) {
          const files = Array.from(event.clipboardData?.files ?? []).filter((f) => f.type.startsWith('image/'));
          if (!files.length) return false;
          event.preventDefault();
          uploaderRef.current(view, files);
          return true;
        },
        drop(event, view) {
          const files = Array.from(event.dataTransfer?.files ?? []).filter((f) => f.type.startsWith('image/') || !f.type);
          if (!files.length) return false;
          event.preventDefault();
          const pos = view.posAtCoords({ x: event.clientX, y: event.clientY }) ?? undefined;
          uploaderRef.current(view, files, pos);
          return true;
        },
      }),
    ],
    // Nhãn/chữ mẫu đổi theo ngôn ngữ giao diện ⇒ dựng lại phần mở rộng (giữ nội dung, lịch sử hoàn tác).
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [label, i18n.language, live],
  );

  useEffect(() => () => onView(null), [onView]);

  return (
    <CodeMirror
      value={value}
      onChange={onChange}
      height="100%"
      className={cx('h-full', invalid && 'ring-2 ring-red-500 ring-inset')}
      theme={dark ? 'dark' : 'light'}
      placeholder={placeholder}
      basicSetup={{
        lineNumbers: false,
        foldGutter: false,
        highlightActiveLineGutter: false,
        highlightActiveLine: false,
        autocompletion: false,
        bracketMatching: true,
        closeBrackets: false,
        searchKeymap: true,
      }}
      extensions={extensions}
      onCreateEditor={(view) => onView(view)}
    />
  );
}

