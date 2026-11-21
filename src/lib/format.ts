// Small display helpers.

export function watts(v: number | undefined | null): string {
  if (v == null || Number.isNaN(v)) return "—";
  const r = Math.round(v);
  return Math.abs(r) >= 1000 ? `${(r / 1000).toFixed(1)} кВт` : `${r} Вт`;
}

export function hm(iso: string): string {
  return iso.slice(11, 16);
}

/** «сьогодні», «завтра» or `dd.mm`. */
export function dayLabel(date: string): string {
  const d = new Date(date + "T00:00:00");
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const diff = Math.round((d.getTime() - today.getTime()) / 86_400_000);
  const dm = `${date.slice(8, 10)}.${date.slice(5, 7)}`;
  if (diff === 0) return `Сьогодні, ${dm}`;
  if (diff === 1) return `Завтра, ${dm}`;
  if (diff === -1) return `Вчора, ${dm}`;
  return dm;
}

export function isToday(iso: string): boolean {
  const now = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return iso.slice(0, 10) === `${now.getFullYear()}-${p(now.getMonth() + 1)}-${p(now.getDate())}`;
}

export const sourceLabel = { battery: "Від станції", grid: "Від мережі", ai: "AI-режим EcoFlow" } as const;
