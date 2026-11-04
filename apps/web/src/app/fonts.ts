// Phông Inter BIẾN THIÊN (@fontsource-variable/inter): một tệp woff2 cho mỗi bảng mã, đủ mọi độ đậm 100–900 — thay
// 5 độ đậm × 3 bảng mã = 15 tệp của @fontsource/inter. Nạp qua next/font/local để có URL băm (cache vĩnh viễn) và
// preload (header `Link` do React/Next gửi) cho tệp chính (latin).
//
// Ba "họ" phông tách theo unicode-range (giống hệt cách CSS của fontsource chia tệp): trình duyệt chỉ tải tệp có ký tự
// cần dùng — trang tiếng Việt cần latin + vietnamese, latin-ext gần như không bao giờ tải. Thứ tự trong --font-sans
// (globals.css): vietnamese, latin-ext, latin rồi mới tới phông dự phòng — phông dự phòng (Arial chỉnh cỡ, do next/font
// tính từ số đo Inter để chữ không nhảy khi đổi phông) chỉ đứng SAU CÙNG, nếu không chữ có dấu sẽ hiện bằng Arial.
//
// Đường dẫn trỏ vào node_modules gốc repo (npm workspaces hoisting — xem package-lock: node_modules/@fontsource-variable/inter).
import localFont from 'next/font/local';

// Tham số của localFont() phải là giá trị CHỮ (trình biên dịch đọc tĩnh) — không dùng biến, spread hay template string.

/** Bảng mã latin cơ bản — tệp chính, preload; kèm phông dự phòng Arial đã chỉnh số đo (chống CLS). */
export const interLatin = localFont({
  src: '../../../../node_modules/@fontsource-variable/inter/files/inter-latin-wght-normal.woff2',
  weight: '100 900',
  style: 'normal',
  display: 'swap',
  variable: '--font-inter-latin',
  preload: true,
  adjustFontFallback: 'Arial',
  declarations: [
    {
      prop: 'unicode-range',
      value:
        'U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD',
    },
  ],
});

/** Chữ cái tiếng Việt có dấu (ă â đ ê ô ơ ư, dấu thanh…) — ~10 KB. Không preload: đo Lighthouse không thấy lợi, tệp nhỏ. */
export const interVietnamese = localFont({
  src: '../../../../node_modules/@fontsource-variable/inter/files/inter-vietnamese-wght-normal.woff2',
  weight: '100 900',
  style: 'normal',
  display: 'swap',
  variable: '--font-inter-vi',
  preload: false,
  adjustFontFallback: false,
  declarations: [
    {
      prop: 'unicode-range',
      value:
        'U+0102-0103,U+0110-0111,U+0128-0129,U+0168-0169,U+01A0-01A1,U+01AF-01B0,U+0300-0301,U+0303-0304,U+0308-0309,U+0323,U+0329,U+1EA0-1EF9,U+20AB',
    },
  ],
});

/** Latin mở rộng (ā ł ş…) — hiếm khi cần, chỉ tải khi trang có ký tự trong bảng này. */
export const interLatinExt = localFont({
  src: '../../../../node_modules/@fontsource-variable/inter/files/inter-latin-ext-wght-normal.woff2',
  weight: '100 900',
  style: 'normal',
  display: 'swap',
  variable: '--font-inter-latin-ext',
  preload: false,
  adjustFontFallback: false,
  declarations: [
    {
      prop: 'unicode-range',
      value:
        'U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF',
    },
  ],
});

/** Gắn lên <html>: đặt 3 biến CSS --font-inter-* mà --font-sans (globals.css) dùng. */
export const fontVariables = `${interLatin.variable} ${interVietnamese.variable} ${interLatinExt.variable}`;
