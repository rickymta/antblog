import { useId } from 'react';
import { useTranslation } from 'react-i18next';

interface RoadNode {
  key: 'basics' | 'database' | 'languages' | 'patterns' | 'architecture' | 'advanced';
  x: number;
  y: number;
  w: number;
}

const H = 46;
const NODES: RoadNode[] = [
  { key: 'basics', x: 30, y: 16, w: 170 },
  { key: 'database', x: 260, y: 100, w: 170 },
  { key: 'languages', x: 0, y: 200, w: 222 },
  { key: 'patterns', x: 250, y: 200, w: 200 },
  { key: 'architecture', x: 140, y: 300, w: 190 },
  { key: 'advanced', x: 270, y: 390, w: 180 },
];

// Đường nối nét đứt: từ đáy thẻ nguồn tới đỉnh thẻ đích (cong bậc ba), mũi tên ở cuối.
const EDGES = [
  'M115 62 C115 92 345 70 345 96',
  'M315 146 C315 178 110 168 110 196',
  'M365 146 C365 172 350 172 350 196',
  'M110 246 C110 278 210 270 210 296',
  'M350 246 C350 278 260 270 260 296',
  'M235 346 C235 376 360 362 360 386',
];

/**
 * Sơ đồ lộ trình trong hero (kiểu A3): thẻ trắng nhỏ nối bằng nét đứt có mũi tên — SVG tự vẽ, không ảnh.
 * Cả sơ đồ là một hình (`role="img"`) có mô tả đầy đủ; chữ bên trong ẩn với trình đọc màn hình.
 */
export function RoadmapDiagram({ className = '' }: { className?: string }) {
  const { t } = useTranslation();
  const uid = useId().replace(/:/g, '');
  const arrowId = `tkh-arrow-${uid}`;
  const gradId = `tkh-final-${uid}`;
  const shadowId = `tkh-shadow-${uid}`;

  return (
    <svg viewBox="0 0 470 450" role="img" aria-label={t('home.roadmapLabel')} className={className}>
      <defs>
        <marker id={arrowId} viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
          <path d="M0 0 L10 5 L0 10 z" fill="#BFDBFE" />
        </marker>
        <linearGradient id={gradId} x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="#6366F1" />
          <stop offset="100%" stopColor="#2563EB" />
        </linearGradient>
        <filter id={shadowId} x="-10%" y="-20%" width="120%" height="160%">
          <feDropShadow dx="0" dy="6" stdDeviation="6" floodColor="#020617" floodOpacity="0.35" />
        </filter>
      </defs>

      <g fill="none" stroke="#BFDBFE" strokeOpacity="0.75" strokeWidth="2" strokeDasharray="6 6" strokeLinecap="round">
        {EDGES.map((d) => (
          <path key={d} d={d} className="tkh-dash-flow" markerEnd={`url(#${arrowId})`} />
        ))}
      </g>

      <g aria-hidden="true" fontFamily="Inter, ui-sans-serif, system-ui, sans-serif">
        {NODES.map((n, i) => {
          const final = n.key === 'advanced';
          return (
            <g key={n.key} filter={`url(#${shadowId})`}>
              <rect x={n.x} y={n.y} width={n.w} height={H} rx={10} fill={final ? `url(#${gradId})` : '#FFFFFF'} />
              <circle cx={n.x + 23} cy={n.y + H / 2} r={13} fill={final ? 'rgba(255,255,255,0.22)' : '#DBEAFE'} />
              <text x={n.x + 23} y={n.y + H / 2 + 4.5} textAnchor="middle" fontSize="12" fontWeight="700" fill={final ? '#FFFFFF' : '#1D4ED8'}>
                {i + 1}
              </text>
              <text x={n.x + 44} y={n.y + H / 2 + 5} fontSize="14" fontWeight="600" fill={final ? '#FFFFFF' : '#0F172A'}>
                {t(`home.roadmap.${n.key}`)}
              </text>
            </g>
          );
        })}
      </g>
    </svg>
  );
}
