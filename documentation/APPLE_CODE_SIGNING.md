# Apple Developer ID signing for BellaNote

This is the full path from paying Apple to a signed, notarized GitHub Release. You create a **Developer ID Application** certificate, make this Mac trust the chain, then put six secrets in GitHub Actions. The Release workflow signs the app, re-signs the Whisper sidecar with its own entitlements, smoke-starts it, then notarizes the `.dmg`.

Paying for the Apple Developer Program does not sign anything by itself. Signing without notarization is also not enough for a browser download. Both happen only after the secrets below exist and you cut a **new** release.

If you still have an unsigned `.dmg` from before those secrets existed, see [`RUNNING_UNSIGNED_VERSIONS.md`](./RUNNING_UNSIGNED_VERSIONS.md). Do not put that workaround in release notes.

Official references: [Apple Developer Program](https://developer.apple.com/programs/), [Certificates, Identifiers & Profiles](https://developer.apple.com/account/resources/certificates/list), [Apple PKI](https://www.apple.com/certificateauthority/), [Tauri macOS code signing](https://v2.tauri.app/distribute/sign/macos/).

---

## What you will have at the end

| GitHub Actions secret | What it is |
| --------------------- | ---------- |
| `APPLE_CERTIFICATE` | One-line base64 of a `.p12` that contains the cert **and** private key |
| `APPLE_CERTIFICATE_PASSWORD` | Password you set when exporting that `.p12` |
| `APPLE_SIGNING_IDENTITY` | Full identity string, e.g. `Developer ID Application: Zachary Watts (VFY2YRW9Z3)` |
| `APPLE_ID` | Apple ID email for the developer team |
| `APPLE_PASSWORD` | An **app-specific** password, not the Apple ID login |
| `APPLE_TEAM_ID` | 10-character Team ID, e.g. `VFY2YRW9Z3` |

Do **not** create empty placeholders. Tauri treats a present `APPLE_CERTIFICATE` as “import this .p12,” even when the value is `""`. GitHub Actions turns a missing secret into an empty string. The workflow only exports `APPLE_*` when `APPLE_CERTIFICATE` is actually set.

Do **not** add a `KEYCHAIN_PASSWORD` secret. The workflow uses a local throwaway password only when `macos_import_signing_cert.sh` creates a job-scoped keychain after Tauri finishes (Tauri’s own keychain is deleted when `tauri-action` exits).

---

## 1. Enroll in the Apple Developer Program

1. Sign in at [developer.apple.com](https://developer.apple.com) with the Apple ID that should own the team.
2. Join the [Apple Developer Program](https://developer.apple.com/programs/) (~$99/year) and finish enrollment (identity check, payment, agreements).
3. When membership is **Active**, open [Account](https://developer.apple.com/account) and copy the 10-character **Team ID**. That is `APPLE_TEAM_ID`.

Until membership is active, Developer ID certificates are not available.

---

## 2. Create a Certificate Signing Request on this Mac

The private key is generated **on this Mac** and stays in Keychain. The `.cer` Apple gives you later is only the public certificate. Install it on the **same Mac** that made the CSR, or you cannot sign.

1. Open **Keychain Access**.
2. **Keychain Access → Certificate Assistant → Request a Certificate From a Certificate Authority…**
3. Fill the form:

   | Field | Value |
   | ----- | ----- |
   | User Email Address | The Apple ID email for this developer team |
   | Common Name | Your name or company name (a label for the key, **not** the app name) |
   | CA Email Address | **Leave blank** |

4. Choose **Saved to disk**, not “Emailed to the CA.”
5. You do not need “Let me specify key pair information.” The default 2048-bit RSA key is what Apple expects.
6. Continue and save the `.certSigningRequest` file.

---

## 3. Create a Developer ID Application certificate

You want **Developer ID Application**. That signs a Mac app for distribution **outside** the Mac App Store (GitHub `.dmg` files).

Do **not** pick:

- Anything under **Services** (Push, Wallet, Apple Pay, Swift packages, WatchKit, VoIP). BellaNote does not use those.
- **Apple Development** — only for your own Mac / Xcode.
- **Apple Distribution** / Mac App Store types.
- **Developer ID Installer** — for `.pkg` installers. BellaNote ships a `.app` inside a `.dmg`.

Steps:

1. Open [Certificates, Identifiers & Profiles](https://developer.apple.com/account/resources/certificates/list) → **+**.
2. Under **Software**, choose **Developer ID Application**. If you only see a Services list, go back and pick Software first.
3. Profile type: **G2 Sub-CA (Xcode 11.4.1 or later)**. Skip “Previous Sub-CA.”
4. Upload the `.certSigningRequest` from step 2.
5. Download the `.cer`. Keep a copy somewhere safe. Apple’s page will look like:

   - Certificate Type: **Developer ID Application**
   - A name such as **Zachary Watts**
   - An expiration about five years out

---

## 4. Install the leaf certificate in Keychain

1. On the **same Mac** that created the CSR, double-click the `.cer`.
2. It should land in the **login** keychain.
3. In Keychain Access, check **My Certificates** (not only the Certificates category). You want an identity named like:

   `Developer ID Application: Zachary Watts (VFY2YRW9Z3)`

   Expand it. A **private key** must sit under the certificate. No private key means the `.cer` was installed on the wrong Mac, or Keychain did not pair it with the CSR key.

The leaf cert often shows **“certificate is not trusted”** in red at this point. That is expected until the G2 intermediate is installed. Do **not** set the leaf to Always Trust.

---

## 5. Trust the Developer ID G2 intermediate

`security find-identity -v -p codesigning` only lists **valid** identities. With the leaf installed but the G2 CA missing, you typically see:

```text
0 valid identities found
```

Without `-v`, the identity is there but invalid:

```bash
security find-identity -p codesigning
```

```text
Matching identities
  1) … "Developer ID Application: Zachary Watts (VFY2YRW9Z3)"
     1 identities found

Valid identities only
     0 valid identities found
```

The leaf is issued by `Developer ID Certification Authority` / `OU=G2`. Install Apple’s intermediate:

1. Download [Developer ID G2 CA](https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer) from [Apple PKI](https://www.apple.com/certificateauthority/) (**Developer ID - G2**).
2. Double-click it, or:

   ```bash
   curl -fsSL -o /tmp/DeveloperIDG2CA.cer \
     https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer
   security add-certificates -k "$HOME/Library/Keychains/login.keychain-db" \
     /tmp/DeveloperIDG2CA.cer
   ```

3. Quit and reopen Keychain Access if the red warning is still showing.
4. Confirm a **valid** identity:

   ```bash
   security find-identity -v -p codesigning
   ```

   You want:

   ```text
   1) … "Developer ID Application: Zachary Watts (VFY2YRW9Z3)"
      1 valid identities found
   ```

Copy that quoted string **exactly**. That is `APPLE_SIGNING_IDENTITY`. It is **not** just the Team ID.

| Secret | Example |
| ------ | ------- |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Zachary Watts (VFY2YRW9Z3)` |
| `APPLE_TEAM_ID` | `VFY2YRW9Z3` |

---

## 6. Export a `.p12` and base64-encode it

GitHub’s Mac runners do not have your Keychain. They need a passworded `.p12` that contains both the certificate and the private key.

1. Keychain Access → **My Certificates** → expand **Developer ID Application: …**
2. Right-click the **private key** → **Export**.
3. Save a `.p12` and set a password you will remember. That password is `APPLE_CERTIFICATE_PASSWORD`. This file is also your backup of public + private key. Keep it off the git repo (not in Downloads long-term).
4. Encode:

   ```bash
   openssl base64 -A -in /path/to/certificate.p12 -out certificate-base64.txt
   ```

   The **one line** in that text file is `APPLE_CERTIFICATE`.

---

## 7. Create an app-specific password

Notarization uses your Apple ID, but **not** your normal login password.

1. Open [appleid.apple.com](https://appleid.apple.com/account/manage) → **App-Specific Passwords**.
2. Create one (label it e.g. `BellaNote notarization`).
3. That value is `APPLE_PASSWORD`.
4. `APPLE_ID` is the Apple ID email, e.g. `Zach@ZachWatts.online`.

---

## 8. Add the six GitHub Actions secrets

In the BellaNote GitHub repo: **Settings → Secrets and variables → Actions → New repository secret**.

Add all six at once:

| Name | Value |
| ---- | ----- |
| `APPLE_CERTIFICATE` | Entire one-line contents of `certificate-base64.txt` |
| `APPLE_CERTIFICATE_PASSWORD` | Password from the `.p12` export |
| `APPLE_SIGNING_IDENTITY` | Full string from `security find-identity -v -p codesigning` |
| `APPLE_ID` | Apple ID email |
| `APPLE_PASSWORD` | App-specific password from step 7 |
| `APPLE_TEAM_ID` | 10-character Team ID |

After the secrets are saved, delete `certificate-base64.txt` from Downloads. Keep a backup of the `.p12` somewhere safe (password manager, encrypted drive). Do not commit either file.

---

## 9. Run the Release workflow

Keep `version` in sync in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` if you are cutting a new version.

The macOS job **must not** notarize inside Tauri. Tauri signs the `.app` (and would stamp the Whisper sidecar with the app’s mic/screen entitlements). A later step re-signs `transcribe-worker` with [`entitlements-sidecar.plist`](../src-tauri/entitlements-sidecar.plist) (`disable-library-validation` only — no unsigned executable memory, JIT, or DYLD exceptions), smoke-starts it, builds the `.dmg` from that `.app`, then notarizes and staples.

The main `bellanote` binary keeps [`entitlements.plist`](../src-tauri/entitlements.plist) (mic + screen capture). It must not gain `disable-library-validation`.

1. **Actions → Release → Run workflow**, or push a tag `v` + that version (example: `v0.1.0-beta.3`).
2. The macOS job is Apple Silicon only (`macos-latest` / `aarch64-apple-darwin`). Intel Macs are not supported.
3. Open **Export Apple signing secrets**. You want that step **not** to log `No Apple Developer certificate`. If it does, the `.app` is unsigned — see [`RUNNING_UNSIGNED_VERSIONS.md`](./RUNNING_UNSIGNED_VERSIONS.md). That step exports the certificate and signing identity only. It does **not** export `APPLE_ID` into the Tauri build (that would notarize before the sidecar is re-signed).
4. Confirm **Re-sign sidecar, smoke, notarize DMG** passes. Failure with `different Team IDs` / `Failed to load Python` means the sidecar still cannot load Python.org’s framework.
5. Wait until both macOS and Windows jobs succeed. Open the **draft** on the repo **Releases** page (not “Create a new release”), download the Apple Silicon `.dmg`, smoke-test it, then publish.

Local `npm run tauri:build` on this Mac runs the same sidecar re-sign and smoke start after Tauri bundles (`scripts/macos_pack.sh`). Set `APPLE_SIGNING_IDENTITY` (and the other Apple env vars if you want a notarized local disk image). GitHub Actions still needs the `.p12` secrets; the runner has no copy of your login keychain.

---

## Troubleshooting

| Symptom | Cause | Fix |
| ------- | ----- | --- |
| Certificate list is only Push / Wallet / Apple Pay | You are on **Services** | Go back; choose **Software** → **Developer ID Application** |
| `.cer` installs but there is no private key | Wrong Mac, or CSR key not in login | Repeat CSR + cert on the Mac that will export the `.p12` |
| Keychain: “certificate is not trusted”; `0 valid identities found` with `-v` | Missing G2 intermediate | Install [DeveloperIDG2CA.cer](https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer) (step 5). Do not Always-Trust the leaf. |
| Release log: `No Apple Developer certificate` | `APPLE_CERTIFICATE` secret empty or unset | Re-paste the one-line base64; do not create empty secrets. Running that `.dmg` is covered in [`RUNNING_UNSIGNED_VERSIONS.md`](./RUNNING_UNSIGNED_VERSIONS.md) |
| `security import` / `SecKeychainItemImport` fails in CI | Empty `APPLE_CERTIFICATE` was exported | Same as above; the workflow is supposed to skip export when the secret is empty |
| Notarization fails | Wrong `APPLE_PASSWORD` (used login password) or wrong Team ID | Use an app-specific password; `APPLE_TEAM_ID` is the 10-character id, not the full identity string |
| Sidecar dies with `different Team IDs` / `Failed to load Python` | Worker signed with Hardened Runtime but without `disable-library-validation` | Re-sign with `scripts/macos_resign_sidecar.sh`; do not add unsigned-executable-memory or JIT |
| Main app has `disable-library-validation` | Sidecar and app plists were mixed | App uses `entitlements.plist` only; worker uses `entitlements-sidecar.plist` |
