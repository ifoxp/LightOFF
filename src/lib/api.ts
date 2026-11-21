// Typed wrappers around the Tauri commands.
import { invoke } from "@tauri-apps/api/core";
import type { Cell, Settings, Snapshot, Source } from "./types";

export const inTauri = "__TAURI_INTERNALS__" in window;

export const api = {
  snapshot: () => invoke<Snapshot>("snapshot"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<void>("save_settings", { settings }),
  testConnection: (settings: Settings) => invoke<string>("test_connection", { settings }),
  refreshStatus: () => invoke<void>("refresh_status"),
  setSource: (to: Source) => invoke<number>("set_source", { to }),
  setReserve: (value: number) => invoke<number>("set_reserve", { value }),
  setAiMode: () => invoke<void>("set_ai_mode"),
  setAutomation: (enabled: boolean) => invoke<void>("set_automation", { enabled }),
  setCells: (date: string, from: number, to: number, cell: Cell) => invoke<void>("set_cells", { date, from, to, cell }),
  clearDay: (date: string) => invoke<void>("clear_day", { date }),
  openVoe: () => invoke<void>("open_voe"),
  importLegacy: () => invoke<Settings | null>("import_legacy"),
  autostartStatus: () => invoke<boolean>("autostart_status"),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  testNotification: () => invoke<void>("test_notification"),
  openDataDir: () => invoke<void>("open_data_dir"),
  quit: () => invoke<void>("quit_app"),
};

export function errText(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
}
