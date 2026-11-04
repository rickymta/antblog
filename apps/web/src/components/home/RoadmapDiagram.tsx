'use client';

import type { CSSProperties } from 'react';
import { useId } from 'react';
import Link from 'next/link';
import { useTranslation } from 'react-i18next';

interface RoadNode {
  key: 'basics' | 'database' | 'languages' | 'patterns' | 'architecture' | 'advanced';
  href: string;
  x: number;
  y: number;
  w: number;
}

// Hệ toạ độ gốc của sơ đồ (viewBox 470 × 450); thẻ là liên kết HTML đặt tuyệt đối theo % nên co giãn cùng nét nối SVG.
const VW = 470;
const VH = 450;
const H = 46;
const NODES: RoadNode[] = [
  { key: 'basics', href: '/articles?level=beginner', x: 30, y: 16, w: 170 },
  { key: 'database', href: '/topics/database', x: 260, y: 100, w: 170 },
  { key: 'languages', href: '/topics/programming', x: 0, y: 200, w: 222 },
  { key: 'patterns', href: '/topics/design-pattern', x: 250, y: 200, w: 200 },
  { key: 'architecture', href: '/topics/architecture', x: 140, y: 300, w: 190 },
  { key: 'advanced', href: '/articles?level=advanced', x: 270, y: 390, w: 180 },
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

const pct = (v: number, total: number) => `${(v / total) * 100}%`;

/**
 * Sơ đồ lộ trình trong hero (kiểu A3): thẻ trắng nhỏ nối bằng nét đứt có mũi tên.
 * Mỗi thẻ là một liên kết thật (Tab tới được, có viền khi focus) dẫn tới bài theo cấp độ hoặc trang chủ đề tương ứng.
 * Kích thước chữ theo `cqw` của khung ⇒ sơ đồ co giãn đồng đều như một hình SVG.
 */
export function RoadmapDiagram({ className = '' }: { className?: string }) {
  const { t } = useTranslation();
  const uid = useId().replace(/[^a-zA-Z0-9_-]/g, '');
  const arrowId = `tkh-arrow-${uid}`;

  return (
    <div role="group" aria-label={t('home.roadmapLabel')} className={`@container relative aspect-[470/450] ${className}`}>
      <svg viewBox={`0 0 ${VW} ${VH}`} aria-hidden="true" focusable="false" className="absolute inset-0 h-full w-full overflow-visible">
        <defs>
          <marker id={arrowId} viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M0 0 L10 5 L0 10 z" fill="#BFDBFE" />
          </marker>
        </defs>
        <g fill="none" stroke="#BFDBFE" strokeOpacity="0.75" strokeWidth="2" strokeDasharray="6 6" strokeLinecap="round">
          {EDGES.map((d) => (
            <path key={d} d={d} className="tkh-dash-flow" markerEnd={`url(#${arrowId})`} />
          ))}
        </g>
      </svg>

      <ol className="absolute inset-0 m-0 list-none p-0">
        {NODES.map((n, i) => {
          const final = n.key === 'advanced';
          const label = t(`home.roadmap.${n.key}`);
          const style: CSSProperties = { left: pct(n.x, VW), top: pct(n.y, VH), width: pct(n.w, VW), height: pct(H, VH) };
          return (
            <li key={n.key} className="absolute" style={style}>
              <Link
                href={n.href}
                aria-label={t('home.roadmapStep', { n: i + 1, label })}
                className={`group/node flex h-full w-full items-center gap-[1.7cqw] rounded-[2.13cqw] pl-[2.13cqw] pr-[1.5cqw] text-[2.98cqw] leading-none font-semibold shadow-[0_6px_14px_rgba(2,6,23,0.35)] transition-[transform,box-shadow] duration-150 hover:-translate-y-0.5 hover:shadow-[0_10px_20px_rgba(2,6,23,0.45)] hover:ring-2 hover:ring-sky-300 focus-visible:-translate-y-0.5 focus-visible:ring-2 focus-visible:ring-sky-300 focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-white motion-reduce:transition-none motion-reduce:hover:translate-y-0 ${
                  final ? 'bg-gradient-to-br from-[#6366F1] to-[#2563EB] text-white' : 'bg-white text-[#0F172A]'
                }`}
              >
                <span
                  aria-hidden="true"
                  className={`inline-flex h-[5.53cqw] w-[5.53cqw] shrink-0 items-center justify-center rounded-full text-[2.55cqw] font-bold ${
                    final ? 'bg-white/20 text-white' : 'bg-[#DBEAFE] text-[#1D4ED8] group-hover/node:bg-[#BFDBFE]'
                  }`}
                >
                  {i + 1}
                </span>
                <span className="truncate">{label}</span>
              </Link>
            </li>
          );
        })}
      </ol>
    </div>
  );
}
