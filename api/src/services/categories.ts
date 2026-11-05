import { Category, type ICategory } from '../models/category.js';
import { validation } from '../lib/errors.js';

type TreeCategory = Pick<ICategory, '_id' | 'parentId'>;

/** Validate the resulting complete graph, including moved descendants. Root depth is 1. */
export function validateCategoryTree(categories: TreeCategory[], id: string, parentId: string | null): void {
  const parents = new Map(categories.map(c => [String(c._id), c.parentId ? String(c.parentId) : null]));
  if (parentId && !parents.has(parentId)) throw validation({ parentId: 'Danh mục cha không tồn tại.' });
  parents.set(id, parentId);
  for (const start of parents.keys()) {
    const seen = new Set<string>();
    let current: string | null = start;
    while (current) {
      if (seen.has(current)) throw validation({ parentId: 'Danh mục không được chứa chính nó hoặc tạo vòng lặp.' });
      seen.add(current);
      if (seen.size > 3) throw validation({ parentId: 'Cây danh mục chỉ được có tối đa 3 cấp, bao gồm cả các danh mục con.' });
      current = parents.get(current) ?? null;
    }
  }
}

export async function validateCategoryParent(id: string, parentId: string | null) {
  const categories = await Category.find({}).select({ _id: 1, parentId: 1 }).lean<ICategory[]>();
  validateCategoryTree(categories, id, parentId);
}

export async function categorySubtreeIds(id: ICategory['_id']) {
  const categories = await Category.find({}).select({ _id: 1, parentId: 1 }).lean<ICategory[]>();
  const found = new Set([String(id)]);
  for (let level = 0; level < 2; level++) {
    for (const c of categories) if (c.parentId && found.has(String(c.parentId))) found.add(String(c._id));
  }
  return categories.filter(c => found.has(String(c._id))).map(c => c._id);
}
