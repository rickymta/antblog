import { randomBytes } from 'node:crypto';
import bcrypt from 'bcryptjs';
import type { Types } from 'mongoose';
import { addDays, vnDate, vnDayStart } from '../src/lib/dates.js';
import {
  Article,
  Comment,
  CommentLike,
  DailyArticleStat,
  DailySiteStat,
  Reaction,
  REACTION_TYPES,
  User,
  type IArticle,
} from '../src/models/index.js';
import { recountArticleActivity, recountCommentLikes } from '../src/services/counters.js';

type Log = (m: string) => void;

/** PRNG có hạt giống để dữ liệu demo ổn định giữa các lần chạy. */
function mulberry32(seed: number) {
  let a = seed;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const DEMO_NAMES = [
  'Nguyễn Minh Anh',
  'Trần Quốc Bảo',
  'Lê Thu Hà',
  'Phạm Đức Huy',
  'Hoàng Gia Linh',
  'Vũ Thành Nam',
  'Đặng Mai Phương',
  'Bùi Tuấn Kiệt',
];

const ROOT_BODIES = [
  'Bài viết rất dễ hiểu, cảm ơn tác giả đã chia sẻ!',
  'Phần ví dụ minh hoạ giúp mình hiểu rõ hơn nhiều so với tài liệu chính thức.',
  'Mình đang áp dụng cách này vào dự án ở công ty, hiệu quả thấy rõ.',
  'Có thể viết thêm phần so sánh hiệu năng giữa các cách tiếp cận không ạ?',
  'Lưu lại để đọc kỹ cuối tuần. Hay quá!',
  'Phần sơ đồ trực quan, đọc một lần là nắm được ý chính.',
];

const REPLY_BODIES = [
  'Đồng ý, mình cũng gặp tình huống tương tự.',
  'Bạn thử thêm index cho trường đó xem, mình làm vậy thì nhanh hơn hẳn.',
  'Cảm ơn bạn, góp ý rất hữu ích!',
  'Mình nghĩ còn tuỳ quy mô dữ liệu nữa.',
  'Chuẩn luôn, mình cũng đang chờ phần tiếp theo.',
];

/** Gỡ mọi dữ liệu demo và tính lại các bộ đếm bị ảnh hưởng. */
export async function removeDemo(log: Log = console.log): Promise<{ users: number; comments: number }> {
  const users = await User.find({ isDemo: true }).select({ _id: 1 }).lean();
  const userIds = users.map((u) => u._id);
  const roots = await Comment.find({ $or: [{ isDemo: true }, { authorId: { $in: userIds } }] })
    .select({ _id: 1, articleId: 1 })
    .lean();
  const rootIds = roots.map((c) => c._id);
  // Phản hồi (kể cả của người thật) nằm dưới bình luận demo cũng bị gỡ theo — không thể tồn tại thiếu gốc.
  const children = await Comment.find({ parentId: { $in: rootIds } }).select({ _id: 1, articleId: 1 }).lean();
  const commentIds = [...rootIds, ...children.map((c) => c._id)];

  const affectedArticles = new Map<string, Types.ObjectId>();
  for (const c of [...roots, ...children]) affectedArticles.set(c.articleId.toString(), c.articleId);
  for (const r of await Reaction.find({ $or: [{ isDemo: true }, { userId: { $in: userIds } }] }).select({ articleId: 1 }).lean()) {
    affectedArticles.set(r.articleId.toString(), r.articleId);
  }

  // Lượt xem demo đã cộng vào `stats.views` ⇒ trừ lại đúng phần đó.
  const demoViews = await DailyArticleStat.aggregate<{ _id: Types.ObjectId; v: number }>([
    { $match: { isDemo: true } },
    { $group: { _id: '$articleId', v: { $sum: '$views' } } },
  ]);
  for (const d of demoViews) {
    await Article.updateOne({ _id: d._id }, { $inc: { 'stats.views': -d.v } });
    await Article.updateOne({ _id: d._id, 'stats.views': { $lt: 0 } }, { $set: { 'stats.views': 0 } });
  }

  const likedByDemo = await CommentLike.find({ userId: { $in: userIds } }).select({ commentId: 1 }).lean();
  await CommentLike.deleteMany({ $or: [{ isDemo: true }, { userId: { $in: userIds } }, { commentId: { $in: commentIds } }] });
  await Reaction.deleteMany({ $or: [{ isDemo: true }, { userId: { $in: userIds } }] });
  await Comment.deleteMany({ _id: { $in: commentIds } });
  await DailyArticleStat.deleteMany({ isDemo: true });
  await DailySiteStat.deleteMany({ isDemo: true });
  await User.deleteMany({ _id: { $in: userIds } });

  await recountArticleActivity([...affectedArticles.values()]);
  await recountCommentLikes(likedByDemo.map((l) => l.commentId));
  log(`[demo] Đã gỡ ${userIds.length} tài khoản demo, ${commentIds.length} bình luận demo và số liệu demo.`);
  return { users: userIds.length, comments: commentIds.length };
}

/** Tạo hoạt động demo (luôn gỡ bản demo cũ trước ⇒ chạy lại không nhân đôi). */
export async function createDemo(log: Log = console.log): Promise<void> {
  await removeDemo(() => undefined);
  const articles = await Article.find({ status: 'published' }).sort({ publishedAt: -1 }).lean<IArticle[]>();
  if (!articles.length) {
    log('[demo] Chưa có bài viết đã xuất bản — chạy `npm run seed` trước.');
    return;
  }
  const rand = mulberry32(20261103);
  const pick = <T>(arr: readonly T[]): T => arr[Math.floor(rand() * arr.length)]!;
  const now = new Date();
  const today = vnDate(now);
  const randomTimeOn = (date: string) => {
    const start = vnDayStart(date).getTime();
    const end = Math.min(start + 24 * 3600 * 1000, now.getTime());
    return new Date(start + rand() * Math.max(1, end - start));
  };

  // ---- Tài khoản demo ----
  const users = [];
  for (let i = 0; i < DEMO_NAMES.length; i++) {
    users.push(
      await User.create({
        email: `demo${i + 1}@demo.local`,
        passwordHash: await bcrypt.hash(randomBytes(18).toString('base64url'), 12),
        displayName: DEMO_NAMES[i],
        role: 'reader',
        status: 'active',
        isDemo: true,
        createdAt: addDaysDate(now, -35 + i),
      }),
    );
  }

  // ---- Lượt truy cập + lượt xem 30 ngày ----
  const popularity = articles.map(() => 0.2 + rand());
  const totalPop = popularity.reduce((s, x) => s + x, 0);
  const siteDocs = [];
  const viewDocs: { date: string; articleId: Types.ObjectId; isDemo: true; views: number }[] = [];
  const viewsByArticle = new Map<string, number>();
  for (let d = 29; d >= 0; d--) {
    const date = addDays(today, -d);
    const weekday = new Date(`${date}T00:00:00Z`).getUTCDay();
    const weekend = weekday === 0 || weekday === 6;
    const visits = Math.round((weekend ? 60 : 110) + rand() * 90 + (29 - d) * 2);
    siteDocs.push({ date, isDemo: true, visits, uniqueVisitors: Math.round(visits * (0.55 + rand() * 0.2)) });
    const dayViews = Math.round(visits * (1.1 + rand() * 0.5));
    articles.forEach((a, i) => {
      const v = Math.round((dayViews * popularity[i]!) / totalPop * (0.6 + rand() * 0.8));
      if (v <= 0) return;
      viewDocs.push({ date, articleId: a._id, isDemo: true, views: v });
      viewsByArticle.set(a._id.toString(), (viewsByArticle.get(a._id.toString()) ?? 0) + v);
    });
  }
  await DailySiteStat.insertMany(siteDocs);
  if (viewDocs.length) await DailyArticleStat.insertMany(viewDocs);
  for (const a of articles) {
    const v = viewsByArticle.get(a._id.toString());
    if (v) await Article.updateOne({ _id: a._id }, { $inc: { 'stats.views': v } });
  }

  // ---- Bình luận 2 cấp ----
  const commentDocs = [];
  const targets = articles.slice(0, Math.min(6, articles.length));
  for (const article of targets) {
    const rootCount = 1 + Math.floor(rand() * 3);
    for (let r = 0; r < rootCount; r++) {
      const rootAuthor = pick(users);
      const rootDate = addDays(today, -Math.floor(rand() * 28) - 1);
      const root = new Comment({
        articleId: article._id,
        authorId: rootAuthor._id,
        body: pick(ROOT_BODIES),
        status: 'visible',
        isDemo: true,
        createdAt: randomTimeOn(rootDate),
      });
      commentDocs.push(root);
      const replyCount = Math.floor(rand() * 3);
      let lastReply: InstanceType<typeof Comment> | undefined;
      for (let k = 0; k < replyCount; k++) {
        const author = pick(users.filter((u) => !u._id.equals(rootAuthor._id)));
        const nextDay = addDays(rootDate, k + 1);
        const replyDate = nextDay <= today ? nextDay : today;
        const reply = new Comment({
          articleId: article._id,
          parentId: root._id,
          // Phản hồi thứ hai trở đi có thể trả lời một phản hồi khác ⇒ ghi người được trả lời.
          replyToUserId: lastReply && rand() < 0.5 ? lastReply.authorId : undefined,
          authorId: author._id,
          body: pick(REPLY_BODIES),
          status: 'visible',
          isDemo: true,
          createdAt: randomTimeOn(replyDate),
        });
        if (reply.createdAt < root.createdAt) reply.createdAt = new Date(root.createdAt.getTime() + 60_000 * (k + 1));
        commentDocs.push(reply);
        lastReply = reply;
      }
    }
  }
  if (commentDocs.length) await Comment.insertMany(commentDocs);

  // ---- Like bình luận ----
  const likeDocs = [];
  for (const c of commentDocs) {
    for (const u of users) {
      if (!u._id.equals(c.authorId) && rand() < 0.3) likeDocs.push({ commentId: c._id, userId: u._id, isDemo: true, createdAt: c.createdAt });
    }
  }
  if (likeDocs.length) await CommentLike.insertMany(likeDocs);

  // ---- Reaction ----
  const reactionDocs = [];
  for (const u of users) {
    for (const a of articles) {
      if (rand() > 0.45) continue;
      const types = REACTION_TYPES.filter(() => rand() < 0.35);
      for (const type of types.length ? types : [pick(REACTION_TYPES)]) {
        reactionDocs.push({ articleId: a._id, userId: u._id, type, isDemo: true, createdAt: randomTimeOn(addDays(today, -Math.floor(rand() * 30))) });
      }
    }
  }
  if (reactionDocs.length) await Reaction.insertMany(reactionDocs);

  await recountArticleActivity(articles.map((a) => a._id));
  await recountCommentLikes(commentDocs.map((c) => c._id));
  log(
    `[demo] Đã tạo ${users.length} tài khoản demo (@demo.local), ${commentDocs.length} bình luận, ${likeDocs.length} lượt thích, ` +
      `${reactionDocs.length} reaction, số liệu truy cập 30 ngày.`,
  );
}

function addDaysDate(d: Date, days: number): Date {
  return new Date(d.getTime() + days * 24 * 3600 * 1000);
}

