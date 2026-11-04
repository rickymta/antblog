import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { ImageResponse } from 'next/og';

// Ảnh Open Graph 1200×630 dựng bằng next/og (Satori). Font Inter (.woff — Satori không đọc woff2) lấy từ @fontsource,
// chép vào apps/web/assets/fonts để có trong bản standalone (next.config: outputFileTracingIncludes).
// Nạp đủ ba bộ ký tự latin + latin-ext + vietnamese ⇒ chữ tiếng Việt có dấu hiển thị đúng.

export const OG_SIZE = { width: 1200, height: 630 };

type Weight = 600 | 800;
// Mỗi bộ ký tự một tên họ font riêng: Satori chọn đúng độ đậm TRONG từng họ rồi mới dò glyph lần lượt theo danh sách fontFamily
// (cùng một tên cho cả ba bộ thì chữ có dấu bị lấy nhầm từ font khác độ đậm).
const SUBSETS = [
  ['latin', 'InterL'],
  ['latin-ext', 'InterLX'],
  ['vietnamese', 'InterVN'],
] as const;
const FONT_FAMILY = 'InterL, InterLX, InterVN';

let fontsPromise: Promise<{ name: string; data: ArrayBuffer; weight: Weight; style: 'normal' }[]> | undefined;

function loadFonts() {
  fontsPromise ??= Promise.all(
    ([600, 800] as const).flatMap((weight) =>
      SUBSETS.map(async ([subset, name]) => {
        const buf = await readFile(path.join(process.cwd(), 'assets/fonts', `inter-${subset}-${weight}-normal.woff`));
        return { name, data: buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength) as ArrayBuffer, weight, style: 'normal' as const };
      }),
    ),
  ).catch((e: unknown) => {
    fontsPromise = undefined;
    throw e;
  });
  return fontsPromise;
}

const BG = 'linear-gradient(135deg, #0B1B4D 0%, #1E3A8A 55%, #312E81 100%)';

function Brand() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 20 }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          width: 72,
          height: 72,
          borderRadius: 16,
          background: 'linear-gradient(135deg, #2563EB 0%, #6366F1 100%)',
          color: 'white',
          fontSize: 30,
          fontWeight: 800,
        }}
      >
        {'</>'}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column' }}>
        <div style={{ fontSize: 34, fontWeight: 800, color: 'white' }}>Tech Knowledge Hub</div>
        <div style={{ fontSize: 22, fontWeight: 600, color: '#BFDBFE' }}>Learn • Share • Grow</div>
      </div>
    </div>
  );
}

/** Khung chung: nền gradient thương hiệu, chấm lưới mờ, logo `</>` + tên site. */
export async function brandImage({ eyebrow, title, footer }: { eyebrow?: string; title: string; footer?: string }) {
  const fonts = await loadFonts();
  const size = title.length > 90 ? 52 : title.length > 60 ? 60 : 70;
  return new ImageResponse(
    (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          padding: '64px 72px',
          background: BG,
          fontFamily: FONT_FAMILY,
          position: 'relative',
        }}
      >
        <div
          style={{
            position: 'absolute',
            top: -160,
            right: -120,
            width: 520,
            height: 520,
            borderRadius: 9999,
            background: 'rgba(99, 102, 241, 0.35)',
            filter: 'blur(80px)',
          }}
        />
        <Brand />
        <div style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
          {eyebrow && (
            <div style={{ display: 'flex', fontSize: 26, fontWeight: 600, color: '#7DD3FC', letterSpacing: 1 }}>{eyebrow}</div>
          )}
          <div style={{ display: 'flex', fontSize: size, fontWeight: 800, color: 'white', lineHeight: 1.12, maxWidth: 1050 }}>{title}</div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontSize: 24, fontWeight: 600, color: '#BFDBFE' }}>
          <div style={{ display: 'flex' }}>{footer ?? ''}</div>
          <div style={{ display: 'flex', height: 8, width: 220, borderRadius: 8, background: 'linear-gradient(90deg, #38BDF8, #6366F1)' }} />
        </div>
      </div>
    ),
    { ...OG_SIZE, fonts },
  );
}
