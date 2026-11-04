import { brandImage, OG_SIZE } from '../lib/og';

// Ảnh chia sẻ mặc định của site (trang không có ảnh riêng). Dựng một lần lúc build.
export const alt = 'Tech Knowledge Hub — Learn • Share • Grow';
export const size = OG_SIZE;
export const contentType = 'image/png';

export default async function Image() {
  return brandImage({
    eyebrow: 'Kho kiến thức kỹ thuật tiếng Việt',
    title: 'Hiểu sâu công nghệ. Xây giải pháp tốt hơn.',
    footer: 'Database · Ngôn ngữ lập trình · Design Pattern · Architecture',
  });
}
