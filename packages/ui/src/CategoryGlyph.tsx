import type { LucideIcon, LucideProps } from 'lucide-react';
import { ClipboardList, Container, Cpu, Database, CodeXml, FlaskConical, Network, Shapes, ShieldCheck } from 'lucide-react';
import type { CategoryIcon } from '@tkh/shared';

const ICONS: Record<CategoryIcon, LucideIcon> = {
  database: Database,
  code: CodeXml,
  pattern: Shapes,
  architecture: Network,
  devops: Container,
  system: Cpu,
  security: ShieldCheck,
  testing: FlaskConical,
  analysis: ClipboardList,
};

/** Biểu tượng của chủ đề (`category.icon`). Trang trí — đặt `aria-hidden`, tên chủ đề luôn đi kèm. */
export function CategoryGlyph({ icon, ...props }: { icon: CategoryIcon } & Omit<LucideProps, 'ref'>) {
  const Icon = ICONS[icon] ?? CodeXml;
  return <Icon aria-hidden="true" {...props} />;
}
