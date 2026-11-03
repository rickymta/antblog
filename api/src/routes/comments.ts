import { Router, type RequestHandler } from 'express';
import { Types } from 'mongoose';
import { z } from 'zod';
import { conflict, forbidden, isDuplicateKeyError, notFound, validation } from '../lib/errors.js';
import { isObjectId, pagingSchema, parse } from '../lib/validate.js';
import { currentUser, requireAuth } from '../middleware/auth.js';
import { Article, Comment, CommentLike, type IArticle, type IComment } from '../models/index.js';
import { softDeleteComment } from '../services/comments.js';
import { toComments, type CommentOut } from '../services/serializers.js';
import { findPublicArticle } from './articles.js';

const emptyToUndefined = (v: unknown) => (v === '' ? undefined : v);

const listSchema = z.object({
  sort: z.preprocess(emptyToUndefined, z.enum(['newest', 'oldest', 'top']).default('newest')),
  ...pagingSchema(10),
});

const bodySchema = z.string().trim().min(1).max(2000);

const createSchema = z.object({
  body: bodySchema,
  parentId: z.preprocess((v) => (v === '' || v === null ? undefined : v), z.string().optional()),
});

const SORTS: Record<'newest' | 'oldest' | 'top', Record<string, 1 | -1>> = {
  newest: { createdAt: -1, _id: -1 },
  oldest: { createdAt: 1, _id: 1 },
  top: { likeCount: -1, createdAt: -1, _id: -1 },
};

const unavailable = () => conflict('COMMENT_UNAVAILABLE', 'Bình luận này đã bị xoá hoặc ẩn.');

async function findComment(id: string): Promise<IComment> {
  if (!isObjectId(id)) throw notFound('Không tìm thấy bình luận.');
  const c = await Comment.findById(id).lean<IComment>();
  if (!c) throw notFound('Không tìm thấy bình luận.');
  return c;
}

export function commentsRouter(createLimiter: RequestHandler): Router {
  const r = Router();

  r.get('/articles/:slug/comments', async (req, res) => {
    const q = parse(listSchema, req.query);
    const article = await findPublicArticle(String(req.params.slug));
    const skip = (q.page - 1) * q.pageSize;
    // Gốc được hiện khi còn `visible`, hoặc (đã xoá/ẩn) nhưng vẫn còn phản hồi đang hiện — trả như `deleted`.
    const [result] = await Comment.aggregate<{ items: IComment[]; total: { n: number }[] }>([
      { $match: { articleId: article._id, parentId: null } },
      {
        $lookup: {
          from: Comment.collection.name,
          localField: '_id',
          foreignField: 'parentId',
          as: 'visibleReplies',
          pipeline: [{ $match: { status: 'visible' } }, { $project: { _id: 1 } }],
        },
      },
      { $addFields: { visibleReplyCount: { $size: '$visibleReplies' } } },
      { $match: { $or: [{ status: 'visible' }, { visibleReplyCount: { $gt: 0 } }] } },
      { $sort: SORTS[q.sort] },
      {
        $facet: {
          items: [{ $skip: skip }, { $limit: q.pageSize }, { $project: { visibleReplies: 0 } }],
          total: [{ $count: 'n' }],
        },
      },
    ]);
    const roots = (result?.items ?? []) as (IComment & { visibleReplyCount: number })[];
    // Phản hồi: hiện bình luận `visible` và chỗ giữ chỗ `deleted`; bình luận bị ẩn không bao giờ trả ra.
    const replies = roots.length
      ? await Comment.find({ parentId: { $in: roots.map((x) => x._id) }, status: { $in: ['visible', 'deleted'] } })
          .sort({ createdAt: 1, _id: 1 })
          .lean<IComment[]>()
      : [];
    const out = await toComments([...roots, ...replies], req.user, 'public');
    const byId = new Map(out.map((c) => [c.id, c]));
    const repliesByRoot = new Map<string, CommentOut[]>();
    for (const rep of replies) {
      const key = rep.parentId!.toString();
      const list = repliesByRoot.get(key) ?? [];
      list.push(byId.get(rep._id.toString())!);
      repliesByRoot.set(key, list);
    }
    const items = roots.map((root) => {
      const id = root._id.toString();
      return { ...byId.get(id)!, replies: repliesByRoot.get(id) ?? [], replyCount: root.visibleReplyCount };
    });
    const fresh = await Article.findById(article._id).select({ 'stats.comments': 1 }).lean<Pick<IArticle, 'stats'>>();
    res.json({
      items,
      total: result?.total[0]?.n ?? 0,
      page: q.page,
      pageSize: q.pageSize,
      commentCount: Math.max(0, fresh?.stats?.comments ?? 0),
    });
  });

  r.post('/articles/:slug/comments', requireAuth, createLimiter, async (req, res) => {
    const input = parse(createSchema, req.body);
    const article = await findPublicArticle(String(req.params.slug));
    const user = currentUser(req);
    let parentId: Types.ObjectId | undefined;
    let replyToUserId: Types.ObjectId | undefined;
    if (input.parentId !== undefined) {
      const invalidParent = () => validation({ parentId: 'Bình luận được trả lời không thuộc bài viết này' });
      if (!isObjectId(input.parentId)) throw invalidParent();
      const parent = await Comment.findById(input.parentId).lean<IComment>();
      if (!parent || !parent.articleId.equals(article._id)) throw invalidParent();
      let root = parent;
      if (parent.parentId) {
        // Trả lời một phản hồi ⇒ gắn vào GỐC của nó, ghi nhận người được trả lời.
        const found = await Comment.findById(parent.parentId).lean<IComment>();
        if (!found) throw invalidParent();
        root = found;
        if (parent.status !== 'visible') {
          throw conflict('PARENT_UNAVAILABLE', 'Bình luận bạn trả lời đã bị xoá hoặc ẩn.');
        }
        replyToUserId = parent.authorId;
      }
      if (root.status !== 'visible') throw conflict('PARENT_UNAVAILABLE', 'Bình luận gốc đã bị xoá hoặc ẩn.');
      parentId = root._id;
    }
    const doc = await Comment.create({
      articleId: article._id,
      parentId,
      replyToUserId,
      authorId: user._id,
      body: input.body,
      status: 'visible',
    });
    await Article.updateOne({ _id: article._id }, { $inc: { 'stats.comments': 1 } });
    const [out] = await toComments([doc.toObject()], user, 'public');
    res.status(201).json(out);
  });

  r.patch('/comments/:id', requireAuth, async (req, res) => {
    const { body } = parse(z.object({ body: bodySchema }), req.body);
    const comment = await findComment(String(req.params.id));
    const user = currentUser(req);
    if (!comment.authorId.equals(user._id)) throw forbidden('Bạn chỉ sửa được bình luận của mình.');
    const updated = await Comment.findOneAndUpdate(
      { _id: comment._id, status: 'visible' },
      { $set: { body, editedAt: new Date() } },
      { new: true },
    ).lean<IComment>();
    if (!updated) throw unavailable();
    const [out] = await toComments([updated], user, 'public');
    res.json(out);
  });

  r.delete('/comments/:id', requireAuth, async (req, res) => {
    const comment = await findComment(String(req.params.id));
    const user = currentUser(req);
    if (!comment.authorId.equals(user._id) && user.role !== 'admin') {
      throw forbidden('Bạn chỉ xoá được bình luận của mình.');
    }
    await softDeleteComment(comment._id);
    res.status(204).end();
  });

  r.put('/comments/:id/like', requireAuth, async (req, res) => {
    const comment = await findComment(String(req.params.id));
    if (comment.status !== 'visible') throw unavailable();
    const user = currentUser(req);
    try {
      await CommentLike.create({ commentId: comment._id, userId: user._id });
      const updated = await Comment.findByIdAndUpdate(comment._id, { $inc: { likeCount: 1 } }, { new: true }).lean<IComment>();
      return void res.json({ likeCount: Math.max(0, updated?.likeCount ?? 0), likedByMe: true });
    } catch (err) {
      if (!isDuplicateKeyError(err)) throw err;
    }
    res.json({ likeCount: Math.max(0, comment.likeCount ?? 0), likedByMe: true });
  });

  r.delete('/comments/:id/like', requireAuth, async (req, res) => {
    const comment = await findComment(String(req.params.id));
    const user = currentUser(req);
    const removed = await CommentLike.deleteOne({ commentId: comment._id, userId: user._id });
    let likeCount = comment.likeCount ?? 0;
    if (removed.deletedCount === 1) {
      const updated = await Comment.findByIdAndUpdate(comment._id, { $inc: { likeCount: -1 } }, { new: true }).lean<IComment>();
      likeCount = updated?.likeCount ?? 0;
    }
    res.json({ likeCount: Math.max(0, likeCount), likedByMe: false });
  });

  return r;
}

