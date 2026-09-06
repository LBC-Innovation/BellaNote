# Running unsigned BellaNote builds

Use this only if you have an **unsigned** `.app` or `.dmg` (a local build before signing was set up, a CI artifact that logged `No Apple Developer certificate`, or an old GitHub draft from before Developer ID secrets existed). Current GitHub Releases are signed and notarized. People installing from a published release do not need these steps.

macOS may say **“BellaNote is damaged and can’t be opened.”** That is Gatekeeper quarantine on an unsigned download, not a broken installer. On current macOS, **right-click → Open** is no longer offered. **Do not move the app to the Trash.**

## After dragging to Applications

```bash
xattr -cr /Applications/BellaNote.app
```

Then open BellaNote normally.

Sharing over AirDrop or a USB stick usually skips quarantine.

## How you can tell a build was unsigned

The Release job **Export Apple signing secrets** logs:

```text
No Apple Developer certificate — building an unsigned .dmg.
```

If that line is missing, the `.dmg` was signed and notarized. Do not run `xattr` “just in case.”

An old unsigned `.dmg` does not become signed later. Download a new release instead.

## Related

- Signing and notarization setup: [`APPLE_CODE_SIGNING.md`](./APPLE_CODE_SIGNING.md)
