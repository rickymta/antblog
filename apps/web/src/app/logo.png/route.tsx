import { ImageResponse } from 'next/og';

// Logo PNG 512×512 (biểu tượng `</>`) cho JSON-LD Organization và web manifest. Dựng một lần lúc build.
export const dynamic = 'force-static';

export function GET() {
  return new ImageResponse(
    (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          borderRadius: 96,
          background: 'linear-gradient(135deg, #2563EB 0%, #6366F1 100%)',
          color: 'white',
          fontSize: 190,
          fontWeight: 700,
          letterSpacing: -6,
        }}
      >
        {'</>'}
      </div>
    ),
    { width: 512, height: 512 },
  );
}
