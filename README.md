<p align="center">
  <img src="src/assets/app-icon.svg" alt="Desktop Timer clock icon" width="128" height="128">
</p>

# Desktop Timer

A macOS menu bar timer with Pomodoro sessions and blurred reminders to move, stretch, and look around.

## Install

Requires macOS 11 or later. The current download supports **Apple Silicon** (M-series Macs).

1. [Download the latest DMG](https://github.com/Khaled2049/desktop-timer/releases/latest).
2. Open it and drag **Desktop Timer** into **Applications**.
3. Launch the app and click its menu bar icon to get started.

Or install from a terminal. Run it again to update:

```sh
curl -fsSL https://raw.githubusercontent.com/Khaled2049/desktop-timer/main/install.sh | sh
```

The app is not notarized. If macOS blocks the downloaded DMG, attempt to open it, then go to **System Settings → Privacy & Security → Open Anyway** if you trust the download. [Apple’s instructions](https://support.apple.com/en-us/102445).

## Use

- **Control + Space** starts, pauses, or resumes the timer from any app.
- **Control + 1**, **2**, or **3** starts a 5, 15, or 25 minute timer.
- **Control + S** opens the timer settings.
- **Control + 0** resets the timer, and **Control + P** turns Pomodoro mode on or off.
- Choose a preset or set a custom countdown.
- **Pomodoro:** 25-minute focus sessions, 5-minute breaks, and a 15-minute break after four sessions.
- When time runs out, a blurred reminder appears with a short guided break, such as an eye rest or a stretch. Use its button or **Escape** to dismiss it. In Pomodoro mode, the button or **Control + Space** starts the next phase.

Closing settings keeps the timer running. Quit from the menu bar to exit.

## Automate

**Focus mode.** macOS does not let apps switch Focus directly, so Desktop Timer runs two shortcuts if you create them in the Shortcuts app:

- **Desktop Timer Focus On** — add a **Set Focus** action that turns a Focus on.
- **Desktop Timer Focus Off** — add a **Set Focus** action that turns it off.

The first runs when a timer or focus session starts. The second runs when it pauses, ends, or you quit. Breaks leave Focus off.

**Shortcuts and scripts.** The installed app answers `desktop-timer://` links. Use them from an **Open URLs** action in Shortcuts, or from a terminal with `open`:

```sh
open desktop-timer://start       # also: pause, toggle, reset
open desktop-timer://preset/10   # start a timer of 1 to 1440 minutes
open desktop-timer://pomodoro    # or timer, to switch modes
open desktop-timer://settings
```

## Develop

Install [Node.js](https://nodejs.org/) 22.12+, [Rust](https://rustup.rs/), and Xcode Command Line Tools (`xcode-select --install`), then run:

```sh
npm ci
npm run tauri dev
```

Build a macOS installer:

```sh
APPLE_SIGNING_IDENTITY="-" npm run tauri build -- --bundles dmg
```

Find the DMG in `src-tauri/target/release/bundle/dmg/`. Allow your terminal to control Finder in **Privacy & Security → Automation** if packaging requests it.

Bug reports and pull requests are welcome. If Desktop Timer helps you take better breaks, [give it a ⭐](https://github.com/Khaled2049/desktop-timer)!
