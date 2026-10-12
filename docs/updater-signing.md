# Updater signing key (1Password + GitHub)

KTXForge’s Tauri updater verifies release artifacts with a minisign key pair.

| Half | Where it lives |
| --- | --- |
| **Public** | Committed in `src-tauri/tauri.conf.json` → `plugins.updater.pubkey` |
| **Private** | 1Password vault **Tech**, item **KTXForge Tauri updater signing key** (source of truth). Copied into GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY` for CI only. |

Never commit the private key. Do **not** keep a copy under `~/.tauri/` (or any other path) on developer machines. Load it from 1Password into the process environment for the duration of a signed local build, then let the shell exit.

The key has **no password**. Do not set `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in GitHub or locally unless you regenerate with a password on purpose.

Losing the private key means existing installs can never receive a signed update. Treat the Tech vault item as a recovery-critical secret.

## Prerequisites (every machine)

1. Access to the Monogrid 1Password account and the **Tech** vault (ask a vault admin if the item is missing).
2. [1Password CLI](https://developer.1password.com/docs/cli/get-started/) (`op`) installed.
3. Desktop 1Password app signed in; enable **Settings → Developer → Integrate with 1Password CLI** so `op` can use the unlocked app (avoids typing the account password into every terminal).
4. Repo checkout with Node LTS and the usual Tauri toolchain (`npm ci` + `npm run setup`).

Confirm CLI access:

```bash
op vault list
op item get "KTXForge Tauri updater signing key" --vault Tech
```

If `op` says you are not signed in and app integration is off:

```bash
eval $(op signin --account monogrid)
```

## Local signed build (Linux / macOS)

Do not write the key to a file. Prefer the repo helper (reads `op://Tech/KTXForge Tauri updater signing key/notesPlain` into the child env only):

```bash
npm run tauri:build:signed
```

Or any command that needs the env:

```bash
npm run with-signing-key -- npm run tauri -- build
```

Override the secret reference if needed:

```bash
export KTXFORGE_UPDATER_KEY_OP_REF='op://Tech/KTXForge Tauri updater signing key/notesPlain'
npm run tauri:build:signed
```

Manual one-shot (same idea, no file):

```bash
export TAURI_SIGNING_PRIVATE_KEY="$(op read 'op://Tech/KTXForge Tauri updater signing key/notesPlain')"
npm run tauri build
unset TAURI_SIGNING_PRIVATE_KEY
```

`npm run tauri dev` does not need the private key. Only builds that produce updater artifacts (`createUpdaterArtifacts`) do.

## Local signed build (Windows)

Install `op` and enable CLI integration in the 1Password app. In PowerShell:

```powershell
npm run tauri:build:signed
```

Or:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = (op read "op://Tech/KTXForge Tauri updater signing key/notesPlain")
npm run tauri build
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY
```

Repeat on each Windows and Linux environment; never copy `*.key` files between PCs.

## Share with tech leads

1. Keep the item in vault **Tech** (already shared with the tech-lead group via vault permissions).
2. Grant new leads **access to the Tech vault** in 1Password (not a separate copy of the key).
3. Point them at this doc. They only need `op` + vault access; they do not initialize a new keypair.

Do not paste the private key into Slack, email, or git. Do not attach key files to the vault item if `notesPlain` already holds the key material.

## CI (GitHub Actions)

`release.yml` sets `TAURI_SIGNING_PRIVATE_KEY` from the repository secret of the same name. Maintainers with `gh` access can refresh it from 1Password without leaving a local file:

```bash
op read 'op://Tech/KTXForge Tauri updater signing key/notesPlain' \
  | gh secret set TAURI_SIGNING_PRIVATE_KEY --repo MONOGRID/KTXForge
```

Delete a password secret if one was created by mistake (this keypair has none):

```bash
gh secret delete TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo MONOGRID/KTXForge || true
```

## Rotate the keypair (only if needed)

Safe before the first public release that users install; after that, rotation orphans existing installs for auto-update.

1. Generate once on a machine that already has vault write access:

   ```bash
   mkdir -p ~/.tauri && chmod 700 ~/.tauri
   npx tauri signer generate -w ~/.tauri/ktxforge.key -p "" --ci -f
   ```

2. Update the Tech vault item (private → `notesPlain`, public → `pubkey` field), or recreate the item with the same title.

3. Replace `plugins.updater.pubkey` in `src-tauri/tauri.conf.json` with the contents of `ktxforge.key.pub` (single line / as printed by the signer).

4. Push the new private key to GitHub (pipe from `op read` or from the temp file once, then delete the file).

5. Remove local key material:

   ```bash
   shred -u ~/.tauri/ktxforge.key ~/.tauri/ktxforge.key.pub 2>/dev/null \
     || rm -f ~/.tauri/ktxforge.key ~/.tauri/ktxforge.key.pub
   ```

6. Commit the pubkey change. Append a short note to [decision-context.md](decision-context.md).

## Item field map

| Field | Purpose |
| --- | --- |
| `notesPlain` | Private key body (password-type field); what `op read` / the helper loads |
| `pubkey` | Public key (documentation; source of truth for the app is still `tauri.conf.json`) |
| `github.secret` | Reminder: `TAURI_SIGNING_PRIVATE_KEY` |
| `repo` | `MONOGRID/KTXForge` |
