# Desktop Timer

A cyberpunk-themed macOS menu bar timer that reminds you to move, stretch, and look around. Includes a simple countdown and Pomodoro sessions, with a global keyboard shortcut and a blurred break reminder.

**Enjoying Desktop Timer? Give this repository a ⭐ on GitHub!** It helps others discover the project.

## Install on macOS

Requires **macOS 11 or later**. Downloaded releases do not require Node.js, Rust, or a terminal.

1. Open this repository’s **Releases** page and download the `.dmg` attached to the latest release.
2. Choose **Apple Silicon** (`aarch64`) for an M-series Mac or **Intel** (`x86_64`) for an Intel Mac. A **Universal** download works on both. You can check your Mac in **Apple menu → About This Mac**.
3. Open the downloaded `.dmg`.
4. Drag **Desktop Timer** into the **Applications** folder shown in the installer.
5. Open **Desktop Timer** from Applications, then eject the installer disk image.

The timer appears in your menu bar. Click it to control the timer or open settings. Closing the settings window keeps the timer running; choose **Quit Desktop Timer** from its menu to exit.

These instructions apply once a release with a DMG has been published. If there are no downloadable release assets yet, use the development instructions below.

### If macOS blocks opening the app

An unsigned or unnotarized release may require an extra step on first launch. After attempting to open the app, go to **System Settings → Privacy & Security** and use **Open Anyway**, if available, only if you trust the download. On older macOS versions, this is under **System Preferences → Security & Privacy**. See [Apple’s instructions for opening downloaded apps](https://support.apple.com/en-us/102445).

## Use

- **Control + S** starts, pauses, or resumes the timer, even while another app is focused or settings are closed.
- Choose a **5-, 15-, or 25-minute preset**, or enter a custom duration from 1 second to 24 hours. Press Start to begin. Menu bar presets start immediately.
- At completion, a blurred overlay covers each connected display and a sound plays. Click **I’m ready** or press **Escape** to dismiss it. Control + S starts a new countdown.
- Native menu bar icons indicate a running timer, pause, completion, or a Pomodoro break.
- Running timers count time spent asleep and finish on wake if their deadline passed. Quitting ends the timer; progress is not saved between launches.

### Pomodoro

Select **Pomodoro** in settings or enable **Pomodoro mode** in the menu bar:

- Focus for **25 minutes**.
- Take a **5-minute break** between focus sessions.
- After four focus sessions, take a **15-minute break**.

The window shows your current phase and completed focus sessions. At the end of each phase, use the reminder’s button or **Control + S** to start the next phase when you’re ready. **Escape** dismisses the reminder without starting it. **Reset cycle** returns to the first focus session. Choosing Timer mode or a custom duration clears Pomodoro progress.

## Development

On a Mac, install [Node.js](https://nodejs.org/) (22.12 or later), the [Rust toolchain](https://rustup.rs/), and Xcode Command Line Tools:

```sh
xcode-select --install
```

Clone this repository, open its directory in your terminal, then run:

```sh
npm ci
npm run tauri dev
```

`npm run dev` previews the interface in a browser; timer controls require the native Tauri app.

Run checks with:

```sh
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

The screen blur uses macOS native vibrancy and Tauri transparent windows. Test reminders on your Mac and with multiple displays; the system’s Reduce Transparency setting can change their appearance.

## Building a downloadable release

Tauri creates a DMG with an app icon and an Applications folder shortcut for drag-and-drop installation. See the [Tauri DMG guide](https://v2.tauri.app/distribute/dmg/).

To build for the architecture of your Mac:

```sh
npm ci
npm run tauri build -- --bundles app,dmg
```

Find the app in `src-tauri/target/release/bundle/macos/` and the installer in `src-tauri/target/release/bundle/dmg/`.

To build a single Universal installer for Apple Silicon and Intel:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm run tauri build -- --target universal-apple-darwin --bundles app,dmg
```

Universal outputs are under `src-tauri/target/universal-apple-darwin/release/bundle/`. See [Tauri’s macOS build guide](https://v2.tauri.app/distribute/macos-application-bundle/).

For a smooth first launch, sign the app with a **Developer ID Application** certificate and notarize it with Apple before publishing. Tauri supports this through its signing configuration and Apple credentials supplied to the build environment. Keep credentials out of the repository. See [Tauri’s signing and notarization guide](https://v2.tauri.app/distribute/sign/macos/).

Create a GitHub Release for the version and attach the DMG as a release asset. GitHub’s automatically generated source ZIP is the source code, not the installable app. Include architecture and signing status in the release notes, and test the downloaded installer on another Mac before sharing it.

## Contributing

Bug reports and pull requests are welcome. For bugs, include your macOS version, whether your Mac is Apple Silicon or Intel, the app version, and steps to reproduce the issue.

If Desktop Timer helps you take better breaks, please **star the repository** and share it with a friend.
