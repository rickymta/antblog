import { Router } from 'express';
import { z } from 'zod';
import { conflict, isDuplicateKeyError, notFound } from '../lib/errors.js';
import { SLUG_PATTERN } from '../lib/text.js';
import { isObjectId, parse } from '../lib/validate.js';
import { LANGUAGES, Term, TERM_STATUSES, type ITerm } from '../models/index.js';
import { toAdminTerm, toGlossaryTerm } from '../services/serializers.js';

/** Độ dài tối đa của định nghĩa ngắn — vừa một hộp thoại nhỏ (đặc tả §Dữ liệu). */
export const SHORT_DEFINITION_MAX = 300;

const languageQuery = z.object({ language: z.preprocess((v) => (v === '' ? undefined : v), z.enum(LANGUAGES).default('vi')) });

/** API công khai: `GET /glossary?language=vi` (danh sách để dò keyword) và `GET /glossary/:slug`. */
export function glossaryRouter(): Router {
  const r = Router();

  r.get('/glossary', async (req, res) => {
    const { language } = parse(languageQuery, req.query);
    const terms = await Term.find({ language, status: 'published' }).sort({ slug: 1 }).lean<ITerm[]>();
    res.json(terms.map(toGlossaryTerm));
  });

  r.get('/glossary/:slug', async (req, res) => {
    const { language } = parse(languageQuery, req.query);
    const term = await Term.findOne({ language, slug: req.params.slug, status: 'published' }).lean<ITerm>();
    if (!term) throw notFound('Không tìm thấy thuật ngữ.');
    res.json({ ...toGlossaryTerm(term), ...(term.body ? { body: term.body } : {}) });
  });

  return r;
}

const trimmedList = z
  .array(z.string().trim().min(1).max(100))
  .max(20)
  .default([])
  .transform((list) => [...new Set(list)]);

const termInputSchema = z.object({
  slug: z.string().trim().toLowerCase().regex(SLUG_PATTERN, 'Slug chỉ gồm chữ thường, số và dấu gạch ngang'),
  language: z.enum(LANGUAGES).default('vi'),
  term: z.string().trim().min(1).max(100),
  english: z.preprocess((v) => (v === '' || v === null ? undefined : v), z.string().trim().max(100).optional()),
  aliases: trimmedList,
  shortDefinition: z.string().trim().min(1).max(SHORT_DEFINITION_MAX),
  body: z.preprocess((v) => (v === '' || v === null ? undefined : v), z.string().max(20_000).optional()),
  caseSensitive: z.boolean().default(false),
  autoLink: z.boolean().default(true),
  status: z.enum(TERM_STATUSES).default('published'),
});

function termId(raw: string) {
  if (!isObjectId(raw)) throw notFound('Không tìm thấy thuật ngữ.');
  return raw;
}

const SLUG_TAKEN = () => conflict('SLUG_TAKEN', 'Slug thuật ngữ đã tồn tại trong ngôn ngữ này.');

/** CRUD cho quản trị viên — gắn dưới `/admin/terms`, sau chốt `adminOnly`. Mọi lần ghi đặt `editedInCms`. */
export function adminTermsRouter(): Router {
  const r = Router();

  r.get('/', async (_req, res) => {
    const terms = await Term.find({}).sort({ language: 1, slug: 1 }).lean<ITerm[]>();
    res.json(terms.map(toAdminTerm));
  });

  r.post('/', async (req, res) => {
    const input = parse(termInputSchema, req.body);
    try {
      const created = await Term.create({ ...input, editedInCms: true });
      res.status(201).json(toAdminTerm(created.toObject()));
    } catch (err) {
      if (isDuplicateKeyError(err)) throw SLUG_TAKEN();
      throw err;
    }
  });

  r.put('/:id', async (req, res) => {
    const id = termId(req.params.id);
    const input = parse(termInputSchema, req.body);
    const unset: Record<string, 1> = {};
    if (input.english === undefined) unset.english = 1;
    if (input.body === undefined) unset.body = 1;
    let updated: ITerm | null;
    try {
      updated = await Term.findByIdAndUpdate(
        id,
        { $set: { ...input, editedInCms: true }, ...(Object.keys(unset).length ? { $unset: unset } : {}) },
        { new: true, runValidators: true },
      ).lean<ITerm>();
    } catch (err) {
      if (isDuplicateKeyError(err)) throw SLUG_TAKEN();
      throw err;
    }
    if (!updated) throw notFound('Không tìm thấy thuật ngữ.');
    res.json(toAdminTerm(updated));
  });

  r.delete('/:id', async (req, res) => {
    const id = termId(req.params.id);
    const deleted = await Term.findByIdAndDelete(id);
    if (!deleted) throw notFound('Không tìm thấy thuật ngữ.');
    res.status(204).end();
  });

  return r;
}
