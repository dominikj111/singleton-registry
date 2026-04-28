# Release Steps

## 1. Prepare the Repository

Run the full check suite — everything must pass cleanly:

```bash
cargo test
cargo check
cargo clippy
cargo fmt --check
```

Run all examples — none should panic:

```bash
cargo run --example basic_usage
cargo run --example trait_contracts
cargo run --example singleton_replacement
```

Then:

- Bump the version in `Cargo.toml`
- Update `README.md`:
  - Installation section: `singleton-registry = "<new-version>"`
  - MSRV badge URL if the minimum Rust version changed
- Add a `CHANGELOG.md` entry for the new version

## 2. Commit and Tag

```bash
git add .
git commit -m "chore: bump version to <new-version>"
```

Create an annotated tag:

```bash
git tag -a v<new-version> -m "Release version <new-version>

- <summary of changes>"
```

Push commits and tag:

```bash
git push origin main
git push origin v<new-version>
```

## 3. Dry Run

```bash
cargo publish --dry-run
```

Fix any issues before proceeding.

## 4. Publish

```bash
cargo publish
```

Requires a crates.io API token (`cargo login` to configure).

---

## Fixing a Misplaced Tag

If you need to move a tag to a different commit:

```bash
git tag -d v<version>
git tag -a v<version> -m "Release version <version>"

git push origin :refs/tags/v<version>
git push origin v<version>
```
