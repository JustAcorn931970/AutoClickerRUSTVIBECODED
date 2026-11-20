# Build OP Clicker 4 with GitHub (no Rust on your PC)

This project is set up so GitHub Actions compiles the Windows app for you. Rust is installed on GitHub's temporary Windows runner, not on your computer.

## One-time setup

1. Sign in at https://github.com/ and create a new repository named `op-clicker-4`.
2. Choose Public or Private. Either works.
3. Upload the **contents** of this project folder into the repository root. Do not upload the outer `op_clicker_4` folder as a nested folder. At the top level you should see:
   - `Cargo.toml`
   - `src/main.rs`
   - `.github/workflows/windows-build.yml`
   - `README.md`
4. Commit the upload to the `main` branch.

## Download the app

1. Open the repository's **Actions** tab.
2. Select **Build OP Clicker 4 for Windows**.
3. Open the newest run. It must show a green check mark for success.
4. At the bottom, under **Artifacts**, download `OP-Clicker-4-Windows-x64`.
5. Extract the downloaded ZIP and run `OP-Clicker-4.exe`.

The artifact contains the standalone Windows x64 app and a README. No Rust installation is needed on your PC. Normal workflow artifacts are retained by GitHub for 30 days.

## Publish a permanent version download

From the repository, open **Releases** → **Draft a new release**, create a new tag such as `v4.0.0`, and publish the release. GitHub Actions will compile the tagged source and attach `OP-Clicker-4-Windows-x64.zip` and `op_clicker_4.exe` to the release.

## What the workflow checks

- Rust formatting (`cargo fmt --check`)
- Project tests (`cargo test`)
- Optimized Windows build (`cargo build --release`)
- Presence of the expected EXE before packaging

A green workflow means these checks and the Windows build completed on GitHub's runner. It does not replace testing the downloaded app on your own Windows PC.
