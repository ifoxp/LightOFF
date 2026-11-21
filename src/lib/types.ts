// Shapes returned by the Rust commands (src-tauri/src/commands.rs).

export type Source = "battery" | "grid";
export type Cell = "on" | "off" | "maybe";

export interface Status {
  soc: number;
  reserve: number;
  min_dsg: number;
  max_chg: number;
  battery_w: number;
  load_w: number;
  grid_w: number;
  pv_w: number;
  self_powered: boolean;
  ai_mode: boolean;
  feed_in: boolean;
  source: "battery" | "grid" | "ai";
  read_at: string;
}

export interface Day {
  date: string;
  cells: Cell[];
}

export interface PlanItem {
  key: string;
  at: string;
  to: Source;
  block: string;
  window: string;
  done: boolean;
  past: boolean;
}

export interface Entry {
  ts: string;
  level: "info" | "ok" | "warn" | "error";
  text: string;
}

export interface Snapshot {
  status: Status | null;
  status_error: string | null;
  days: Day[];
  queue: string | null;
  schedule_source: string;
  schedule_updated: string | null;
  plan: PlanItem[];
  automation: boolean;
  outage_now: boolean;
  next_change: string | null;
  voe_open: boolean;
  voe_stage: string | null;
  voe_visible: boolean;
  has_api: boolean;
  has_address: boolean;
  journal: Entry[];
}

export interface Settings {
  access_key: string;
  secret_key: string;
  sn: string;
  api_host: string;
  city: string;
  street: string;
  house: string;
  lead_minutes: number;
  back_minutes: number;
  block_minutes: number;
  battery_reserve: number | null;
  grid_reserve: number | null;
  include_maybe: boolean;
  notifications: boolean;
  schedule_refresh_minutes: number;
}
