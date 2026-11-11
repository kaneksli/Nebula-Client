# Build Nebula VPN for Windows on GitHub

1. Upload the **contents** of this folder to the root of a GitHub repository (do not upload the outer `vpn-client` folder as an extra nesting level).
2. Open **Actions**. If prompted, enable GitHub Actions.
3. Select **Build Windows EXE** and click **Run workflow** (or push a commit to `main`/`master`).
4. When the run succeeds, open it and download the `nebula-vpn-windows-installers` artifact.

The workflow builds NSIS `.exe` and MSI installers.

## Important runtime note

This prepares the project for CI compilation. The app expects a `sing-box` executable at `%APPDATA%\\com.nebula.vpnclient\\bin\\sing-box.exe` (the exact app-config path is resolved by Tauri). This archive does not bundle that executable. Until a compatible sing-box binary is installed/provisioned and tested, a successful installer build does not guarantee that VPN connections work. TUN mode also requires the appropriate Windows TUN driver and elevated permissions.
