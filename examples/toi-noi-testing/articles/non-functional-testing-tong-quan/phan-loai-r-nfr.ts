/**
 * Lời giải bài thực hành TEST-018: gán R-NFR-01…07 của SPEC vào đặc tính chất lượng ISO/IEC 25010:2023.
 * `chinh` = đặc tính › đặc tính con chính; `phu` = đặc tính liên quan; `null` = không phải yêu cầu đạt/không đạt.
 */
export const ISO25010_2023 = [
  'Functional suitability',
  'Performance efficiency',
  'Compatibility',
  'Interaction capability',
  'Reliability',
  'Security',
  'Maintainability',
  'Flexibility',
  'Safety',
] as const;

export type Characteristic = (typeof ISO25010_2023)[number];

export interface NfrMapping {
  chinh: { dacTinh: Characteristic; dacTinhCon: string[] } | null;
  phu: Characteristic[];
}

export const PHAN_LOAI: Record<string, NfrMapping> = {
  'R-NFR-01': { chinh: { dacTinh: 'Reliability', dacTinhCon: ['availability'] }, phu: [] },
  'R-NFR-02': { chinh: { dacTinh: 'Performance efficiency', dacTinhCon: ['time behaviour'] }, phu: [] },
  'R-NFR-03': { chinh: { dacTinh: 'Performance efficiency', dacTinhCon: ['time behaviour'] }, phu: ['Compatibility'] },
  'R-NFR-04': { chinh: { dacTinh: 'Performance efficiency', dacTinhCon: ['time behaviour'] }, phu: [] },
  'R-NFR-05': { chinh: { dacTinh: 'Reliability', dacTinhCon: ['fault tolerance'] }, phu: ['Compatibility', 'Functional suitability'] },
  'R-NFR-06': { chinh: { dacTinh: 'Security', dacTinhCon: ['confidentiality', 'resistance'] }, phu: [] },
  'R-NFR-07': { chinh: null, phu: ['Performance efficiency'] },
};
