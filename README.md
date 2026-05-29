# MicMuteOverlay

A lightweight Windows utility that toggles your microphone mute with a global
hotkey and shows a crisp, always-on-top overlay reflecting the mute state.
Runs from the system tray.

This is a ground-up rewrite in **Rust** of the original .NET/WinForms app
(preserved under [`legacy-dotnet/`](legacy-dotnet/)). It keeps every feature of
the original while being dramatically smaller and lighter:

- **Single, self-contained `.exe`** (~400 KB) - no .NET, no Visual C++ runtime,
  no installer. Unzip and run.
- **Tiny footprint** - a few MB of RAM; the overlay only repaints when the mute
  state changes, so there is no per-frame cost and no FPS impact on games.
- **No administrator rights required.**
- **Reliable always-on-top.** The overlay uses a layered top-most window and
  re-asserts `HWND_TOPMOST` on a timer (default every 2.5 s), so it no longer
  falls behind other windows over time.
- **True per-pixel alpha** text rendering (anti-aliased outline, no color-key
  fringing) via GDI+ and `UpdateLayeredWindow`.

> Targets borderless/windowed games. Like Discord/Steam overlays, it cannot
> appear over *exclusive* full-screen applications without graphics-API
> injection, which is intentionally out of scope.

## Features

- Global hotkey to toggle microphone mute (default **Ctrl+Shift+M**)
- Microphone picker (lists active capture devices; falls back to the default
  communications device)
- Transparent, borderless, always-on-top overlay text
- Overlay display modes: **When Muted**, **When Unmuted**, **Always**, **Never**
- Customisable text, font size (8-72), text colour, outline colour, outline
  thickness (0-10)
- Mute / unmute sound effects (configurable `.wav` paths)
- Drag the overlay to reposition; **Reset Overlay Position** from the tray
- Click-through mode (overlay ignores the mouse)
- Hotkey recorder in settings (change the hotkey live, no restart)
- System tray menu: Settings, Reset Overlay Position, Exit
- Right-click the overlay for Settings / Exit; double-click the tray icon for Settings
- Start with Windows (per-user registry `Run` entry)
- Settings persist in `config.json`

## Download & run

1. Build the release zip (see below) or grab a packaged `MicMuteOverlay.zip`.
2. Unzip anywhere.
3. Double-click `MicMuteOverlay.exe`.

A `config.json` is created next to the executable on first run (or under
`%APPDATA%\MicMuteOverlay\` if the program folder is read-only). The `Sounds`
folder beside the executable holds the mute/unmute `.wav` files.

To quit, use **Exit** from the tray icon or the overlay's right-click menu.

## Building from source

Requires the [Rust toolchain](https://rustup.rs/) (stable).

```powershell
# Debug build
cargo build

# Optimised single binary
cargo build --release
# -> target\release\micmuteoverlay.exe
```

### Produce a distributable zip

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1
```

This builds the release binary and assembles `dist\MicMuteOverlay\`
(`MicMuteOverlay.exe`, `Sounds\`, `config.json`) plus `dist\MicMuteOverlay.zip`.

## Configuration (`config.json`)

The file is backward-compatible with the original app. Keys:

| Key | Type | Default | Notes |
|-----|------|---------|-------|
| `Hotkey` | string | `"Ctrl+Shift+M"` | e.g. `Ctrl+Alt+F1`, `Shift+M` |
| `OverlayText` | string | `"Mic Muted"` | |
| `MuteSound` | string | `"Sounds/mute.wav"` | played on mute |
| `UnmuteSound` | string | `"Sounds/unmute.wav"` | played on unmute |
| `FontSize` | int | `20` | 8-72 |
| `ForeColor` | string | `"Red"` | named colour |
| `SelectedMicrophoneId` | string | `""` | empty = default device |
| `StartWithWindows` | bool | `false` | per-user `Run` key |
| `OutlineColor` | string | `"Black"` | named colour |
| `OutlineThickness` | int | `2` | 0-10 |
| `DisplayMode` | int | `0` | 0=WhenMuted, 1=WhenUnmuted, 2=Always, 3=Never |
| `ClickThroughMode` | bool | `false` | overlay ignores the mouse |
| `TopMostRefreshMs` | int | `2500` | how often top-most is re-asserted |
| `OverlayX` / `OverlayY` | int | (auto) | saved overlay position |

Named colours: Red, White, Black, Blue, Green, Yellow, Orange, Purple, Pink,
Cyan, Magenta, Gray, DarkRed, DarkBlue, DarkGreen, Navy, Maroon.

## Project layout

```
src/
  main.rs            entry point + message loop
  app.rs             central state + window procedure
  overlay.rs         layered top-most window + UpdateLayeredWindow + topmost timer
  text_render.rs     GDI+ outlined text -> premultiplied DIB
  audio.rs           Core Audio mute control + device enumeration
  hotkey.rs          global hotkey registration + parsing
  hotkey_recorder.rs low-level keyboard hook for the settings recorder
  keys.rs            key-name <-> virtual-key mapping
  tray.rs            system tray icon
  settings.rs        settings window (Win32 controls)
  sound.rs           WAV playback
  colors.rs          named colour -> ARGB
  config.rs          config model, load/save, startup registry
  util.rs            small Win32 string helpers
assets/              icon + bundled sounds
scripts/             release packaging
legacy-dotnet/       the original .NET/WinForms implementation (reference)
```
