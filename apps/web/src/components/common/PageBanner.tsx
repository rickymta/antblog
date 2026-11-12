import { BookOpen, Compass, Route } from 'lucide-react';

const themes = {
  articles: { icon: BookOpen, background: 'linear-gradient(115deg, #172554, #0f172a)', accent: '#93c5fd' },
  paths: { icon: Route, background: 'linear-gradient(115deg, #134e4a, #0f172a)', accent: '#5eead4' },
  about: { icon: Compass, background: 'linear-gradient(115deg, #3b2670, #0f172a)', accent: '#c4b5fd' },
};

/** Decorative vector artwork stays separate from the localized, accessible heading. */
export function PageBanner({ variant, title, subtitle, compact = false }: {
  variant: keyof typeof themes;
  title: string;
  subtitle: string;
  /** Trang danh sách dài: banner thấp, bỏ ô biểu tượng và ẩn hình minh hoạ trên màn hẹp để nội dung lên sớm. */
  compact?: boolean;
}) {
  const { icon: Icon, background, accent } = themes[variant];
  return (
    <header className={`relative isolate overflow-hidden rounded-card bg-slate-900 text-white ${compact ? 'my-4 sm:mb-6' : 'my-6 sm:mb-8'}`} style={{ background }}>
      <div className={`relative z-10 max-w-2xl px-6 sm:px-8 lg:max-w-[58%] ${compact ? 'py-6 sm:py-7' : 'pt-8 sm:pt-10 lg:py-10'}`}>
        {!compact && <span className="mb-5 inline-flex h-12 w-12 items-center justify-center rounded-xl bg-white/10 ring-1 ring-white/20">
          <Icon aria-hidden="true" className="h-6 w-6" style={{ color: accent }} />
        </span>}
        <h1 className={`font-extrabold tracking-tight ${compact ? 'text-2xl sm:text-3xl' : 'text-3xl sm:text-4xl'}`}>{title}</h1>
        <p className={`max-w-xl leading-relaxed text-white/85 ${compact ? 'mt-2 text-base' : 'mt-3 text-base sm:text-lg'}`}>{subtitle}</p>
      </div>
      <svg aria-hidden="true" focusable="false" viewBox="0 0 480 280" className={compact ? 'hidden lg:absolute lg:inset-y-0 lg:right-0 lg:block lg:h-full lg:w-[36%]' : 'ml-auto h-40 w-full max-w-lg sm:h-48 lg:absolute lg:inset-y-0 lg:right-0 lg:h-full lg:w-[42%]'} fill="none">
        <circle cx="260" cy="140" r="118" fill={accent} opacity=".05" />
        <circle cx="260" cy="140" r="92" stroke={accent} strokeOpacity=".15" strokeDasharray="4 8" />
        {variant === 'articles' && <g stroke={accent} strokeWidth="2">
          <rect x="90" y="58" width="145" height="180" rx="12" fill="#172554" transform="rotate(-10 90 58)" />
          <rect x="190" y="34" width="178" height="212" rx="14" fill="#172554" />
          <rect x="210" y="55" width="138" height="65" rx="7" fill={accent} fillOpacity=".12" />
          <path d="m235 77-12 11 12 11m87-22 12 11-12 11m-37-28-12 40" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
          <path d="M212 144h130m-130 20h98m-98 20h118m-118 20h75" strokeOpacity=".6" strokeLinecap="round" />
          <circle cx="374" cy="210" r="27" fill="#1e40af" />
          <path d="M362 210h24m-12-12v24" strokeLinecap="round" />
        </g>}
        {variant === 'paths' && <g stroke={accent} strokeWidth="2.5">
          <path d="M83 217h95q28 0 28-28v-58q0-28 28-28h94q28 0 28-28V48" strokeDasharray="7 7" />
          {([[83,217],[206,161],[275,103],[356,48]] as const).map(([x,y], i) => <g key={i}>
            <circle cx={x} cy={y} r="23" fill="#134e4a" />
            {i < 3 ? <path d={`m${x-8} ${y} 6 6 11-13`} strokeLinecap="round" strokeLinejoin="round" />
              : <path d={`M${x-6} ${y+10}v-22l17 5-17 6`} strokeLinejoin="round" />}
          </g>)}
          <path d="M263 193h107m-107 16h77m-77 16h92" strokeOpacity=".3" strokeLinecap="round" />
        </g>}
        {variant === 'about' && <g stroke={accent} strokeWidth="2">
          <path d="M250 91 130 62m120 29 117-29M250 91v118m0 0-120-47m120 47 117-47" strokeOpacity=".45" />
          <rect x="190" y="85" width="120" height="94" rx="18" fill="#3b2670" />
          <path d="m228 116-15 16 15 16m44-32 15 16-15 16m-17-39-12 44" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
          <circle cx="130" cy="62" r="25" fill="#3b2670" /><path d="M119 62h22m-11-11v22" />
          <rect x="340" y="37" width="54" height="45" rx="10" fill="#3b2670" /><path d="m350 82 2 10 13-10m-13-29h29m-29 12h20" />
          <path d="M106 151q12-7 24 0 12-7 24 0v29q-12-7-24 0-12-7-24 0Zm24 0v29" fill="#3b2670" />
          <circle cx="367" cy="162" r="24" fill="#3b2670" /><path d="m356 162 8 8 15-17" />
          <circle cx="250" cy="219" r="10" fill={accent} />
        </g>}
      </svg>
    </header>
  );
}
