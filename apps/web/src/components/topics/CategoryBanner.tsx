import { CategoryGlyph } from '@tkh/ui';
import { pickLocalized, type Category, type Locale } from '@tkh/shared';
import { bannerBackground } from './bannerBackground';

/**
 * Đầu trang chủ đề (Server Component, không JS phía trình duyệt): banner riêng của chủ đề + biểu tượng, tên, mô tả, số bài.
 * Chữ trắng cố định ở cả hai theme (nền luôn tối). Lớp phủ tối bảo đảm tương phản ≥ 4.5:1: màn hẹp phủ đều và đậm
 * (hình minh hoạ nằm ngay sau chữ), từ `lg` trở lên chỉ đậm bên trái, nơi đặt chữ (đã giới hạn bề ngang), và nhạt dần về phía hình.
 */
export function CategoryBanner({
  category,
  locale,
  eyebrow,
  meta,
}: {
  category: Category;
  locale: Locale;
  eyebrow: string;
  /** Dòng số liệu dưới mô tả (đã định dạng). */
  meta: string;
}) {
  const description = pickLocalized(category.description, locale);
  return (
    <header
      className="relative isolate flex min-h-52 items-center overflow-hidden rounded-card bg-slate-900 sm:min-h-60"
      style={bannerBackground(category.slug, category.color)}
    >
      <div aria-hidden="true" className="absolute inset-0 bg-slate-950/80 lg:bg-slate-950/0 lg:bg-linear-to-r lg:from-slate-950/90 lg:via-slate-950/70 lg:via-55% lg:to-slate-950/10" />
      <div className="relative flex w-full flex-col gap-4 px-5 py-8 sm:flex-row sm:items-center sm:gap-6 sm:px-8 sm:py-10">
        <span className="inline-flex h-14 w-14 shrink-0 items-center justify-center rounded-card bg-white/15 ring-1 ring-white/30 sm:h-16 sm:w-16">
          <CategoryGlyph icon={category.icon} className="h-8 w-8 text-white sm:h-9 sm:w-9" />
        </span>
        <div className="min-w-0 max-w-xl space-y-2 lg:max-w-md">
          <p className="text-xs font-bold tracking-wide text-white/85 uppercase">{eyebrow}</p>
          <h1 className="text-3xl font-extrabold tracking-tight text-white sm:text-4xl">{pickLocalized(category.name, locale)}</h1>
          {description && <p className="text-base leading-relaxed text-white/90">{description}</p>}
          <p className="text-sm font-semibold text-white">{meta}</p>
        </div>
      </div>
    </header>
  );
}
