'use client';

import { useState } from 'react';

const PALETTE = ['#2563EB', '#4F46E5', '#047857', '#7C3AED', '#BE123C', '#B45309', '#0F766E', '#0369A1'];

/** Chữ cái đầu của từ đầu và từ cuối ("Nguyễn Văn An" → "NA"). */
export function initialsOf(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return '?';
  const first = words[0]!.charAt(0);
  const last = words.length > 1 ? words[words.length - 1]!.charAt(0) : '';
  return (first + last).toUpperCase();
}

function colorOf(name: string): string {
  let h = 0;
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) | 0;
  return PALETTE[Math.abs(h) % PALETTE.length]!;
}

/**
 * Avatar tròn: ảnh nếu có và tải được, không thì chữ cái đầu trên nền màu suy từ tên (chữ trắng, tương phản ≥ 4.5:1).
 * Mặc định là phần trang trí (`alt=""`) vì tên người luôn đứng cạnh; truyền `label` khi avatar đứng một mình.
 */
export function Avatar({
  name,
  src,
  size = 36,
  label,
  className = '',
}: {
  name: string;
  src?: string;
  size?: number;
  label?: string;
  className?: string;
}) {
  const [failedSrc, setFailedSrc] = useState<string | null>(null);
  const showImage = Boolean(src) && failedSrc !== src;
  const style = { width: size, height: size, fontSize: Math.max(10, Math.round(size * 0.38)) };
  const base = `inline-flex shrink-0 select-none items-center justify-center overflow-hidden rounded-full ${className}`;

  if (showImage) {
    return (
      <img
        src={src}
        alt={label ?? ''}
        width={size}
        height={size}
        loading="lazy"
        referrerPolicy="no-referrer"
        onError={() => setFailedSrc(src ?? null)}
        className={`${base} bg-surface-2 object-cover`}
        style={style}
      />
    );
  }
  return (
    <span
      className={`${base} font-semibold text-white`}
      style={{ ...style, backgroundColor: colorOf(name) }}
      role={label ? 'img' : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
    >
      {initialsOf(name)}
    </span>
  );
}
