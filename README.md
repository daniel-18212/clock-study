# Clock Study

[Português (Brasil)](README.pt-BR.md)

An offline desktop Pomodoro, countdown timer, and stopwatch with a resizable floating flip clock.

## Download and install

Download your package from [GitHub Releases](https://github.com/daniel-18212/clock-study/releases).

| System | Package | Installation |
| --- | --- | --- |
| Windows 11 x64 | `*-setup.exe` | Run the installer. It includes the offline WebView2 installer for systems that need it. |
| macOS, Apple Silicon and Intel | `*.dmg` | Open the disk image and drag Clock Study into Applications. |
| Ubuntu / Debian x64 | `*.deb` | Open it with your package manager or run `sudo apt install ./<file>.deb` from the download folder. |
| Compatible Linux x64 | `*.AppImage` | Make the file executable and open it. Some distributions require FUSE support. |

Preview releases have no trusted Windows publisher signature or Apple notarization. Windows Smart App Control or macOS Gatekeeper may block them. On Linux Wayland, the compositor controls whether the floating window stays above other apps.

## Use Clock Study

- **Pomodoro:** Set focus, both breaks, and the number of focus blocks before the long break.
- **Countdown:** Set hours, minutes, and seconds. Click the digits to edit the duration.
- **Stopwatch:** Start, pause, resume, reset, and record laps.
- **Floating window:** Click **Flutuar** to compact the window without interrupting the timer. Drag or resize it with the native window controls. Click **Expandir** to restore it.

Durations and window sizes are saved locally. The interface is currently in Brazilian Portuguese.

## Build and release

Install Node.js 22, Rust, and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your system.

```sh
npm install
npm run tauri dev
```

Build on the target OS using `npm run tauri build -- --bundles nsis` for Windows, `--bundles dmg` for macOS, or `--bundles deb,appimage` for Linux.

[Clock Study Desktop](.github/workflows/desktop.yml) builds all three platforms. A version change on `main` publishes the installers and SHA-256 checksums as a prerelease once every build succeeds. Bump the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`, and add bilingual notes under `docs/releases/`. Update both READMEs together when installation, usage, or build steps change.

## Microsoft Store

Create an MSIX product in [Partner Center](https://partner.microsoft.com/dashboard). Set the repository Actions variables `STORE_IDENTITY_NAME`, `STORE_PUBLISHER`, and `STORE_PUBLISHER_DISPLAY_NAME` using **Product identity**. Run **Clock Study Desktop** with **Prepare an MSIX for Microsoft Store submission** selected. Microsoft signs the MSIX after certification.
