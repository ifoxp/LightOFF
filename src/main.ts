// Fonts are bundled locally (no network at runtime): src/fonts.css -> @fontsource woff2 files.
import "./fonts.css";
import "./app.css";

import { mount } from "svelte";
import App from "./App.svelte";
import { applyTheme, loadTheme } from "./lib/theme";

applyTheme(loadTheme());

const app = mount(App, { target: document.getElementById("app")! });

export default app;
