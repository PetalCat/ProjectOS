# Releasing ProjectOS

How to cut a release and how to enable auto-updates.

## Cutting a release

1. **Bump the version** in three places (they must agree):
   - `package.json` → `"version"`
   - `src-tauri/tauri.conf.json` → `"version"`
   - `src-tauri/Cargo.toml` → `[package].version`

2. **Commit the bump** on `main` (e.g. `chore: bump version to 1.0.1`).

3. **Tag and push:**
   ```bash
   git tag v1.0.1
   git push origin main v1.0.1
   ```

4. The `Release` workflow (`.github/workflows/release.yml`) fires on
   the tag push and builds a draft release with installers for:
   - `windows-x64` — `.msi`
   - `macos-arm64` — `.dmg` (Apple Silicon)
   - `macos-x64` — `.dmg` (Intel)
   - `linux-x64` — `.AppImage` + `.deb`

5. Once the workflow completes, open the draft release on GitHub,
   write user-facing release notes, and publish.

For a dry run without tagging, use the workflow's
**Run workflow** button (workflow_dispatch) on a branch.

## Adding auto-updates (post-1.0)

The release workflow already wires the `TAURI_SIGNING_PRIVATE_KEY`
env vars; they're inert until you actually enable the updater plugin.
One-time setup:

1. **Generate the signing keypair** locally:
   ```bash
   pnpm tauri signer generate -w ~/.tauri/projectos.key
   ```
   It prints a public key. Keep the private key file in a password
   manager too — losing it means you can't sign updates.

2. **Add the public key** to `src-tauri/tauri.conf.json` under
   `plugins.updater.pubkey`. Also configure the endpoint:
   ```json
   {
     "plugins": {
       "updater": {
         "pubkey": "<your generated pubkey>",
         "endpoints": [
           "https://github.com/PetalCat/ProjectOS/releases/latest/download/latest.json"
         ]
       }
     }
   }
   ```

3. **Add the updater plugin** to the Rust side:
   - Add `tauri-plugin-updater = "2"` to `src-tauri/Cargo.toml`
     under `[dependencies]`.
   - Register it in `src-tauri/src/lib.rs`:
     ```rust
     tauri::Builder::default()
         .plugin(tauri_plugin_updater::Builder::new().build())
         // ... rest of setup
     ```

4. **Add GitHub Actions secrets** in repo Settings → Secrets and
   variables → Actions:
   - `TAURI_SIGNING_PRIVATE_KEY` — paste the contents of
     `~/.tauri/projectos.key`.
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — the password you set
     when generating the key (or empty string if none).

5. **Wire the frontend.** In Settings, add a "Check for updates"
   button that calls `@tauri-apps/plugin-updater`'s `check()`. On a
   match, prompt the user, then `update.downloadAndInstall()`.

6. **Tag the next release.** `tauri-action` will detect the signing
   env vars and produce signed artifacts plus a `latest.json`
   updater manifest. From that release on, installed apps will see
   the update prompt.

## Code signing (optional)

For truly clean installs (no Gatekeeper / SmartScreen warnings):

- **macOS:** Apple Developer ID. Add `APPLE_CERTIFICATE`,
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`,
  `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` secrets. tauri-action
  picks these up automatically for notarization.
- **Windows:** an Authenticode certificate. Add `WINDOWS_CERTIFICATE`
  and `WINDOWS_CERTIFICATE_PASSWORD`. Self-signed certs work for
  testing but trip SmartScreen on user machines.

Both are optional; v1.0 ships unsigned. macOS users right-click →
Open the first time; Windows users click "More info → Run anyway"
on SmartScreen.
