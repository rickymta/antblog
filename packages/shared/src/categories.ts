import type { Category, Locale } from './types';
import { pickLocalized } from './format';

export interface CategoryTreeNode extends Category { children: CategoryTreeNode[] }

/** Stable sibling order; legacy roots, missing parents and corrupt cycles remain safe to render. */
export function categoryTree(categories: Category[]): CategoryTreeNode[] {
  const sorted = [...categories].sort((a,b) => a.order - b.order || a.slug.localeCompare(b.slug));
  const visited = new Set<string>();
  const ids = new Set(categories.map(c => c.id));
  function branch(c: Category, depth: number): CategoryTreeNode {
    visited.add(c.id);
    return { ...c, children: depth < 3 ? sorted.filter(n => n.parentId === c.id && !visited.has(n.id)).map(n => branch(n, depth + 1)) : [] };
  }
  const roots = sorted.filter(c => !c.parentId || !ids.has(c.parentId)).map(c => branch(c, 1));
  for (const c of sorted) if (!visited.has(c.id)) roots.push(branch(c, 1));
  return roots;
}

export function categoryOptions(categories: Category[], locale: Locale) {
  const output: { category: Category; depth: number; label: string }[] = [];
  function visit(nodes: CategoryTreeNode[], parents: string[]) {
    for (const node of nodes) {
      const names = [...parents, pickLocalized(node.name, locale)];
      output.push({ category: node, depth: names.length, label: names.join(' › ') });
      visit(node.children, names);
    }
  }
  visit(categoryTree(categories), []);
  return output;
}

export function categoryDescendantIds(categories: Category[], id: string): Set<string> {
  const result = new Set([id]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const c of categories) if (c.parentId && result.has(c.parentId) && !result.has(c.id)) { result.add(c.id); changed = true; }
  }
  return result;
}
