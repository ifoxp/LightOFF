// App-wide reactive state: the latest snapshot (polled), the current screen and toasts.
import { api, errText, inTauri } from "./api";
import type { Snapshot } from "./types";

export type Screen = "status" | "schedule" | "journal" | "settings";

interface Toast {
  id: number;
  text: string;
  tone: "ok" | "error";
}

class Store {
  snap = $state<Snapshot | null>(null);
  screen = $state<Screen>("status");
  toasts = $state<Toast[]>([]);
  busy = $state<string | null>(null);
  private nextId = 1;
  private timer = 0;

  start() {
    if (!inTauri) return;
    void this.refresh();
    this.timer = window.setInterval(() => void this.refresh(), 2500);
  }

  stop() {
    clearInterval(this.timer);
  }

  async refresh() {
    try {
      this.snap = await api.snapshot();
    } catch (e) {
      console.error(e);
    }
  }

  toast(text: string, tone: "ok" | "error" = "ok") {
    const id = this.nextId++;
    this.toasts = [...this.toasts, { id, text, tone }];
    setTimeout(() => this.dismiss(id), tone === "error" ? 8000 : 4000);
  }

  dismiss(id: number) {
    this.toasts = this.toasts.filter((t) => t.id !== id);
  }

  /** Run an action with a busy flag; errors become a toast. */
  async run<T>(label: string, f: () => Promise<T>, ok?: (v: T) => string): Promise<T | undefined> {
    this.busy = label;
    try {
      const v = await f();
      if (ok) this.toast(ok(v));
      await this.refresh();
      return v;
    } catch (e) {
      this.toast(errText(e), "error");
      return undefined;
    } finally {
      this.busy = null;
    }
  }
}

export const store = new Store();
