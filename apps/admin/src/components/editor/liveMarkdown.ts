import { syntaxTree } from '@codemirror/language';
import { Decoration, EditorView, ViewPlugin, WidgetType, type DecorationSet, type ViewUpdate } from '@codemirror/view';
import type { Range } from '@codemirror/state';

class InlineImage extends WidgetType {
  constructor(readonly url: string, readonly alt: string) { super(); }
  eq(other: InlineImage) { return this.url === other.url && this.alt === other.alt; }
  toDOM(view: EditorView) {
    const image = document.createElement('img');
    image.src = this.url;
    image.alt = this.alt;
    image.loading = 'lazy';
    image.className = 'cm-article-image';
    image.addEventListener('load', () => view.requestMeasure(), { once: true });
    image.addEventListener('mousedown', event => {
      event.preventDefault();
      // Put the caret inside the Markdown so its source becomes editable.
      view.dispatch({ selection: { anchor: view.posAtDOM(image) + 1 } });
      view.focus();
    });
    return image;
  }
}

const marks: Record<string, string> = {
  StrongEmphasis: 'strong', Emphasis: 'em', Strikethrough: 'strike', InlineCode: 'code', Link: 'link',
};
const inlineTags: Record<string, string> = { u: 'underline', mark: 'highlight', sup: 'sup', sub: 'sub' };

function decorate(view: EditorView) {
  const { state } = view;
  const ranges: Range<Decoration>[] = [];
  const hide = (from: number, to: number) => {
    if (from < to) ranges.push(Decoration.replace({}).range(from, to));
  };
  const mark = (from: number, to: number, kind: string) => {
    if (from < to) ranges.push(Decoration.mark({ class: 'cm-article-' + kind }).range(from, to));
  };
  // Reveal syntax at a caret or selection boundary, including keyboard navigation.
  const active = (from: number, to: number) => state.selection.ranges.some(r =>
    (r.head >= from && r.head <= to) || (r.anchor >= from && r.anchor <= to));
  for (const { from, to } of view.visibleRanges) {
    syntaxTree(state).iterate({ from, to, enter(ref) {
      const node = ref.node;
      const start = node.from, end = node.to;
      if (node.name === 'FencedCode' || node.name === 'CodeBlock') return false;
      if (/^ATXHeading[1-6]$/.test(node.name)) {
        mark(start, end, 'h' + node.name.slice(-1));
        const prefix = node.firstChild;
        if (prefix && !active(start, prefix.to)) hide(start, prefix.to + Number(state.sliceDoc(prefix.to, prefix.to + 1) === ' '));
      }
      if (node.name === 'Image' || node.name === 'Link') {
        const destination = node.getChild('URL');
        const close = node.getChildren('LinkMark')[1];
        if (!destination || !close || state.doc.lineAt(start).number !== state.doc.lineAt(end).number) return;
        if (node.name === 'Image') {
          const url = state.sliceDoc(destination.from, destination.to).replace(/^<|>$/g, '');
          if (!active(start, end) && /^(?:https?:\/\/|\/(?!\/)|\.\.?\/)/i.test(url)) {
            const alt = state.sliceDoc(node.firstChild!.to, close.from);
            ranges.push(Decoration.replace({ widget: new InlineImage(url, alt) }).range(start, end));
          }
          return false;
        }
        mark(start, close.from, 'link');
        if (!active(start, end)) {
          hide(start, node.firstChild!.to);
          hide(close.from, end);
        }
      } else if (marks[node.name]) {
        mark(start, end, marks[node.name]!);
        for (let child = node.firstChild; child; child = child.nextSibling) {
          if (child.name.endsWith('Mark') && !active(child.from, child.to)) hide(child.from, child.to);
        }
      } else if (node.name === 'HTMLTag') {
        const tag = /^<(u|mark|sup|sub)>$/i.exec(state.sliceDoc(start, end))?.[1]?.toLowerCase();
        if (!tag) return;
        for (let sibling = node.nextSibling; sibling; sibling = sibling.nextSibling) {
          if (sibling.name !== 'HTMLTag') continue;
          const token = state.sliceDoc(sibling.from, sibling.to).toLowerCase();
          if (token === '<' + tag + '>') break;
          if (token !== '</' + tag + '>') continue;
          mark(end, sibling.from, inlineTags[tag]!);
          if (!active(start, end)) hide(start, end);
          if (!active(sibling.from, sibling.to)) hide(sibling.from, sibling.to);
          break;
        }
      }
    } });
  }
  return Decoration.set(ranges, true);
}

export const liveMarkdown = [
  ViewPlugin.fromClass(class {
    decorations: DecorationSet;
    constructor(view: EditorView) { this.decorations = decorate(view); }
    update(update: ViewUpdate) {
      if (update.docChanged || update.selectionSet || update.viewportChanged ||
        syntaxTree(update.startState) !== syntaxTree(update.state)) this.decorations = decorate(update.view);
    }
  }, { decorations: plugin => plugin.decorations }),
  EditorView.theme({
    '& .cm-scroller': { fontFamily: 'inherit', fontSize: '1rem' },
    '.cm-article-strong': { fontWeight: 'bold' },
    '.cm-article-em': { fontStyle: 'italic' },
    '.cm-article-strike': { textDecoration: 'line-through' },
    '.cm-article-underline, .cm-article-link': { textDecoration: 'underline' },
    '.cm-article-link': { color: 'var(--color-primary)' },
    '.cm-article-highlight': { backgroundColor: '#fde68a', color: '#422006' },
    '.cm-article-sup': { verticalAlign: 'super', fontSize: '.75em' },
    '.cm-article-sub': { verticalAlign: 'sub', fontSize: '.75em' },
    '.cm-article-code': { fontFamily: 'monospace', background: 'var(--color-surface-2)' },
    '.cm-article-h1, .cm-article-h2, .cm-article-h3, .cm-article-h4, .cm-article-h5, .cm-article-h6': { fontWeight: '700' },
    '.cm-article-h1': { fontSize: '1.8em' },
    '.cm-article-h2': { fontSize: '1.5em' },
    '.cm-article-h3': { fontSize: '1.25em' },
    '.cm-article-image': { display: 'inline-block', maxWidth: '100%', maxHeight: '18rem', cursor: 'text', verticalAlign: 'middle' },
  }),
];
