**English** · [Українська](README.uk.md)

<p align="center">
  <img src="src-tauri/icons/128x128.png" width="96" alt="">
</p>

# LightOFF

A Windows tray app that switches an EcoFlow STREAM between battery and grid
according to the power-outage schedule of Vinnytsiaoblenergo (VOE), the
electricity distributor of the Vinnytsia region in Ukraine.

Rust and Tauri 2, with a Svelte 5 interface. The interface is in Ukrainian.

## The idea worth explaining

### Switching a source the device has no command for

The STREAM has no "run the house from the battery now" command. LightOFF gets
the same effect through the backup reserve (`cfgBackupReverseSoc`) in
Self-powered mode:

| Mode | Reserve written | What happens |
|---|---|---|
| From the station | discharge limit + 3 % (13 % by default), or your own value | the battery covers the house load |
| From the grid | the current charge rounded up, or your own value | the battery holds its charge, the house runs from the grid |

The unit does not accept a reserve below "discharge limit + 3 %", so every
value is clamped between that and the charge limit. If the unit is not in
Self-powered mode, LightOFF turns it on first. The STREAM reacts to the change
in about one to two minutes.

### Outages are planned in two-hour blocks

VOE may announce an outage of four or six hours and then cut only one of its
two-hour parts. So every outage window is split into blocks (two hours by
default), and each block gets its own pair of switches:

- 5 minutes before the block starts: from the station, plus a Windows notification
- 15 minutes after the block starts: back to the grid

Two hours give one pair, four hours two pairs, six hours three. A window that
crosses midnight is treated as one window. All three numbers are editable in
Settings, and "possible outage" cells can be counted as outages or ignored.

An action still runs if it is up to 10 minutes late (sleep, restart, a network
hiccup); a failed one is retried at most once a minute. Actions that already
ran are remembered across restarts.

### Reading a site that sits behind a bot check

`disconnection.voe.com.ua` is behind a Cloudflare check, so LightOFF does not
scrape it over HTTP. It opens the site in its own WebView2 window, as a person
would. An injected script (`voe.js`) searches the saved address through the
site's own search and reads the schedule table. If the site asks for an
"I am not a robot" check, you pass it in that window.

The page gets no access to the app. To hand the result back, the script
navigates to `https://lightoff.invalid/<kind>#<json>`; the app cancels that
navigation and reads the payload from the URL.

Background refreshes (hourly by default) use a hidden window. If the site
wants a check or the address search fails, the hidden window is closed and you
get a notification asking you to refresh from the Schedule screen.

## What it does

- **STREAM status**: charge, reserve, battery, house, grid and solar power
- **Manual switching**: from the station, from the grid, a custom reserve, or
  back to AI mode — from the window or the tray menu
- **VOE schedule** from the site, as described above
- **Manual schedule**: half-hour cells for today and tomorrow; click or drag
  with an "outage", "possible" or "power on" brush
- **Automation** by the schedule, on/off from the window or the tray
- **Journal** of every action, light and dark theme
- **Tray tooltip** with the charge, the current source and the next planned switch

Closing the window hides it to the tray; automation keeps running until you
choose **Quit** in the tray menu.

## What it connects to

- **EcoFlow IoT Open Platform** (`api-e.ecoflow.com` by default; `api.ecoflow.com`
  and `api-a.ecoflow.com` can be chosen in Settings). Requests are signed with
  HMAC-SHA256. Status is read every 20 seconds while the window is open and
  every 60 seconds while it is hidden. Nothing is sent until the keys and the
  serial number are entered.
- **disconnection.voe.com.ua**, in the app's own browser window. The background
  refresh runs only once an address is saved, and can be turned off (interval 0).

There are no other connections.

## Credentials and data

You need an **Access Key** and a **Secret Key** from the EcoFlow developer
platform and the serial number of your STREAM. For the VOE schedule you enter
your city, street and house.

Everything is kept in `%APPDATA%\LightOFF\`:

| File | Contents |
|---|---|
| `settings.json` | EcoFlow keys, serial number, address, automation settings — **plain text** |
| `schedule.json` | the schedule |
| `state.json` | whether automation is on, and which switches already ran |
| `journal.jsonl` | the journal |
| `logs\` | technical logs, kept for 7 days |

Keys from the older Python version of LightOFF can be imported from its
`config.json` in Settings.

## Autostart

Off by default. The **Autostart** switch in Settings adds a `LightOFF` value
under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` that starts the app
minimised to the tray (`--tray`); turning the switch off removes it.

## Building

Needs Rust (stable), Node.js 22+ and pnpm. The VOE window uses WebView2.

```powershell
pnpm install
pnpm tauri dev                  # development with hot reload of the interface
pnpm tauri build                # NSIS installer in src-tauri\target\release\bundle\nsis\
pnpm check                      # svelte-check
```

Logic tests — planner, EcoFlow request signing, reserves, schedule parsing:

```powershell
cd src-tauri
cargo test
```

Builds: see Releases.

```
src/                     interface (Svelte 5)
  lib/screens/           Status, Schedule, Journal, Settings
  lib/components/        buttons, switches, window title bar
src-tauri/src/
  ecoflow.rs             EcoFlow API client (HMAC-SHA256 signing, status, reserve)
  planner.rs             schedule -> outage windows -> blocks -> switches
  automation.rs          background loops: status, running switches, schedule refresh
  schedule.rs            schedule model, storage, parsing the site's data
  voe.rs, voe.js         the VOE site window and the script that reads the table
  tray.rs                tray icon and menu
  autostart.rs           the HKCU Run value
  settings.rs, state.rs  settings, app state, journal
```

## License

License: not specified yet.

---

**Chekaliuk Dmytro** · [@ifoxp](https://github.com/ifoxp) ·
[ifoxp.top](https://ifoxp.top) · Telegram [@ifoxp](https://t.me/ifoxp) ·
Discord `ifoxp`
