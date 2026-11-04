import { ImageResponse } from 'next/og';

// apple-touch-icon 180×180 (iOS "Thêm vào Màn hình chính") — Next tự gắn <link rel="apple-touch-icon">. Dựng một lần lúc build.
// iOS tự bo góc ⇒ nền phủ kín, không bo/không trong suốt.
export const size = { width: 180, height: 180 };
export const contentType = 'image/png';

export default function AppleIcon() {
  return new ImageResponse(
    (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: 'linear-gradient(135deg, #2563EB 0%, #6366F1 100%)',
          color: 'white',
          fontSize: 66,
          fontWeight: 700,
          letterSpacing: -2,
        }}
      >
        {'</>'}
      </div>
    ),
    size,
  );
}
