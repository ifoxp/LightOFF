// Stroke icon set (24×24, stroke 1.75, round caps/joins), same drawing rules as Kermo.
// Unknown key -> `dot`. Every shape is a path so <Icon> renders without {@html}.

const c = (cx: number, cy: number, r: number) =>
  `M${cx - r} ${cy}a${r} ${r} 0 1 0 ${2 * r} 0a${r} ${r} 0 1 0 ${-2 * r} 0`;

export const ICONS: Record<string, string[]> = {
  battery: ["M4 7h14a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1z", "M22 10.5v3", "M6.5 10v4", "M9.5 10v4"],
  plug: ["M9 3v5", "M15 3v5", "M6 8h12v3a6 6 0 0 1-12 0z", "M12 17v4"],
  bolt: ["M13 3L5 13.5h6L10 21l8-10.5h-6z"],
  sun: [
    c(12, 12, 4),
    "M12 2.5v2",
    "M12 19.5v2",
    "M2.5 12h2",
    "M19.5 12h2",
    "M5.3 5.3l1.4 1.4",
    "M17.3 17.3l1.4 1.4",
    "M5.3 18.7l1.4-1.4",
    "M17.3 6.7l1.4-1.4",
  ],
  home: ["M4 11l8-6.5 8 6.5", "M6 9.5V20h12V9.5", "M10 20v-5h4v5"],
  grid: ["M12 3v18", "M7 7l5-4 5 4", "M5 12h14", "M7 17h10", "M8 21l4-6 4 6"],
  calendar: ["M5 5.5h14a1 1 0 0 1 1 1V19a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6.5a1 1 0 0 1 1-1z", "M4 10h16", "M8 3.5v4", "M16 3.5v4"],
  list: ["M9 6h11", "M9 12h11", "M9 18h11", "M4.5 6h.01", "M4.5 12h.01", "M4.5 18h.01"],
  timer: [c(12, 13, 7.5), "M12 13V9.5", "M10 2.5h4", "M18.5 5.5L20 4"],
  settings: ["M4 7h9", "M17 7h3", "M4 17h3", "M11 17h9", "M15 5v4", "M9 15v4"],
  refresh: ["M20 11a8 8 0 0 0-14.6-4.5", "M4 4v4h4", "M4 13a8 8 0 0 0 14.6 4.5", "M20 20v-4h-4"],
  external: ["M14 4h6v6", "M20 4l-9 9", "M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5"],
  folder: ["M3.5 6.5a1 1 0 0 1 1-1h4.6l2 2h8.4a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1h-15a1 1 0 0 1-1-1z"],
  download: ["M12 4v11", "M7.5 10.5L12 15l4.5-4.5", "M5 15v4a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-4"],
  trash: ["M4 7h16", "M10 4h4", "M6 7l1 12a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-12", "M10 11v5", "M14 11v5"],
  bell: ["M6 16V11a6 6 0 0 1 12 0v5l1.5 2h-15z", "M10 20.5a2 2 0 0 0 4 0"],
  close: ["M6 6l12 12", "M18 6L6 18"],
  check: ["M5 12.5l4.5 4.5L19 7.5"],
  warning: ["M10.3 4.3L2.6 18a2 2 0 0 0 1.7 3h15.4a2 2 0 0 0 1.7-3L13.7 4.3a2 2 0 0 0-3.4 0z", "M12 9.5v4", "M12 17h.01"],
  error: [c(12, 12, 9), "M12 7.5v5", "M12 16h.01"],
  info: [c(12, 12, 9), "M12 11v5", "M12 8h.01"],
  power: ["M12 3v8", "M7.05 6.05a7 7 0 1 0 9.9 0"],
  minimize: ["M6 12h12"],
  to_tray: ["M12 4v10", "M8 10l4 4 4-4", "M5 19h14"],
  moon: ["M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z"],
  system_theme: ["M4 4.5h16a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-10a1 1 0 0 1 1-1z", "M12 4.5v12", "M8 20h8"],
  dot: [c(12, 12, 3)],
};

export function iconPaths(key: string): string[] {
  return ICONS[key] ?? ICONS.dot;
}
