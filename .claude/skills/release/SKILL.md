---
name: release
description: Build release binaries for macOS arm64 and Linux amd64, create a GitHub release with artifacts, and update the landing page
disable-model-invocation: true
argument-hint: [version]
allowed-tools: Bash(cargo *), Bash(zip *), Bash(rm *), Bash(git *), Bash(gh *), Bash(unzip *), Bash(file *), Bash(ls *), Bash(export *), Read, Edit, Glob, Grep
---

Build and publish a new release of ghostty-config at version $ARGUMENTS.

## Steps

### 1. Build release binaries

Build for both platforms:

```
# macOS arm64 (native)
cargo build --release

# Linux amd64 (cross-compile via zigbuild, use rustup toolchain)
export PATH="$HOME/.cargo/bin:$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo zigbuild --release --target x86_64-unknown-linux-gnu
```

Prerequisites for cross-compilation:
- `cargo-zigbuild` (`cargo install cargo-zigbuild`)
- `zig` (`brew install zig`)
- rustup target `x86_64-unknown-linux-gnu` (`rustup target add x86_64-unknown-linux-gnu`)

### 2. Create release zips

Package each platform zip with: binary + `static/js/*` + `README.md` + `LICENSE`

```
ghostty-config-v$ARGUMENTS-darwin-arm64.zip
ghostty-config-v$ARGUMENTS-linux-amd64.zip
```

Use `zip -j` for flat files (binary, README, LICENSE), then `zip` to add `static/js/` with directory structure.

Verify with `unzip -l` and `file` (confirm ELF x86-64 for Linux binary).

### 3. Update Cargo.toml version

Update the `version` field in `Cargo.toml` to match `$ARGUMENTS` if it doesn't already.

### 4. Create git tag and GitHub release

```
git tag v$ARGUMENTS
git push origin v$ARGUMENTS
gh release create v$ARGUMENTS \
  ghostty-config-v$ARGUMENTS-darwin-arm64.zip \
  ghostty-config-v$ARGUMENTS-linux-amd64.zip \
  --title "v$ARGUMENTS" \
  --notes "..."
```

Release notes should include: features summary, platforms list (macOS arm64, Linux amd64), and usage instructions.

### 5. Update landing page

Add or verify download/release links in `docs/index.html` hero button group pointing to `https://github.com/madeye/ghostty-config/releases`.

### 6. Commit and push

Commit any changed files (landing page, Cargo.toml) and push to main.

### 7. Clean up

Remove the local zip files after successful upload.

### 8. Verify

Run `gh release view v$ARGUMENTS` to confirm the release and assets.
