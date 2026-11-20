# OP Clicker 4.0 — Rust Edition

A Windows-first desktop auto-clicker with separate Minecraft, Roblox, and plain-white profiles. The app uses normal OS-level mouse input; it does not install a driver, kernel component, or game modification.

## Included profiles

- **Basic White** — the simple, plain-white clicker view.
- **Minecraft Keybind Clicker** — use a chosen hotkey to start/stop or hold-to-click.
- **Minecraft TPS Clicker (Recommended)** — click timing follows a user-set 1–20 TPS target. The default is 20 TPS and one click per tick (20 CPS). It does not read live server TPS from Minecraft.
- **Roblox Auto Clicker** — regular adjustable click rate, up to 30 CPS.
- **Roblox Keybind Clicker** — hotkey-controlled clicking for Roblox.

All profiles can choose left, right, or middle mouse button. The default hotkey is F6. Available hotkeys are F6–F12, X, C, and V.

## Build the app with GitHub (no Rust installation on your PC)

The repository includes **GitHub Actions** in `.github/workflows/windows-build.yml`. GitHub provides a temporary Windows build machine, installs Rust there, checks formatting, runs tests, compiles the app, and uploads the Windows x64 executable.

1. Create a repository on https://github.com/ (Public or Private both work).
2. Upload the **contents** of this `op_clicker_4` folder to the repository's top level. The repository root must contain `Cargo.toml`, `src/`, and `.github/`.
3. Commit the files to the `main` branch.
4. Open the repository's **Actions** tab and select **Build OP Clicker 4 for Windows**. Wait for the run to finish successfully.
5. Open the completed run and find **Artifacts** at the bottom. Download **OP-Clicker-4-Windows-x64**, extract the ZIP, then run `OP-Clicker-4.exe`.

You do **not** need to install Rust, Cargo, or Visual Studio Build Tools on your own PC for this GitHub build. GitHub installs Rust on its hosted Windows runner. Each normal build artifact is available for 30 days.

To publish a versioned download on the repository's **Releases** page, create and publish a GitHub Release with a tag such as `v4.0.0`. The same workflow builds the app and attaches the ZIP and EXE to that release.

## Build locally (optional)

If you prefer building on your own PC, extract the folder and double-click **`build_and_run.bat`**. On the first run, it installs the official Rust stable toolchain for your Windows user if Rust is missing, then downloads dependencies and builds the app. The Windows MSVC toolchain may also require **Visual Studio Build Tools** with the **Desktop development with C++** workload. This local route is optional when using GitHub Actions.

## Using it in a game

1. Choose a profile from **Select variety** in the top-right.
2. Pick the mouse button, hotkey, and activation mode.
3. Put the mouse cursor over the game window, then start the clicker. There is a 350 ms start grace period to give you time to move away from the app's own buttons. With Toggle mode, press the hotkey once to start and again to stop. With Hold mode, keep the hotkey held to click.
4. Use **Open Game Overlay** to make a compact always-on-top controller. Click **Close Overlay / Full Settings** to return to the main window.

The overlay is an ordinary topmost utility window, not a transparent game HUD. It may not appear over exclusive-fullscreen games; use borderless-windowed mode. While the clicker is running, keep the cursor over the game and away from the overlay and other controls, because clicks are sent at the current cursor position.

## Safety and limits

- Normal CPS profiles are capped at 30 CPS. The TPS profile is capped at 40 CPS (20 TPS × up to two clicks per tick).
- The input worker sleeps between polls, waits 350 ms before the first click after activation, and shuts down when the app closes.
- The clicker does not request administrator access. It cannot guarantee that every game or anti-cheat system will accept simulated input. Follow each game's rules and terms.
- Do not leave it running when you are not actively using it. Use the hotkey or Stop button to stop immediately.

## Developer checks

From this folder, run (install rustfmt first with `rustup component add rustfmt` if needed):

```bat
cargo fmt --check
cargo test
cargo check
cargo build --release
```

Settings are stored in `%APPDATA%\OPClicker4\settings.json`.
