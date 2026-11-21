// Theme: follows prefers-color-scheme unless the user forces light/dark (persisted locally).
export type ThemeChoice = "system" | "light" | "dark";

const KEY = "lightoff.theme";

export function loadTheme(): ThemeChoice {
  try {
    const v = localStorage.getItem(KEY);
    if (v === "light" || v === "dark" || v === "system") return v;
  } catch {
    /* storage unavailable: fall back to system */
  }
  return "system";
}

export function applyTheme(choice: ThemeChoice): void {
  const root = document.documentElement;
  if (choice === "system") delete root.dataset.theme;
  else root.dataset.theme = choice;
}

export function saveTheme(choice: ThemeChoice): void {
  applyTheme(choice);
  try {
    localStorage.setItem(KEY, choice);
  } catch {
    /* ignore */
  }
}
