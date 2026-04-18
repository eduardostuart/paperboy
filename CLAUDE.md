# CLAUDE.md

Paperboy is a CLI that fetches RSS, Atom, and JSON Feed subscriptions, filters entries from the last 24 hours, and emails them via SMTP as a Handlebars-rendered digest.

## Crates

- `crates/paperboy/` — library. Feed loading, Handlebars rendering, lettre-based SMTP delivery. Re-exports `Paperboy`, `Feed`, `Entry`, `FeedLoader`, `Mailer`, `Config`, `Credentials`.
- `crates/paperboy-cli/` — binary. Parses env vars, builds `MailConfig`, invokes `Deliver`. `paperboy --version` reads this crate's `Cargo.toml`; this is the version that tracks the release tag scheme.

## How it works end-to-end

1. `subscriptions::load_from_file` reads a newline-separated text file; blank lines and lines starting with `#` are skipped, others are trimmed.
2. `FeedLoader::load` fetches every URL concurrently (up to 10 in flight via `buffer_unordered(10)`), parses through `feed-rs` (handles RSS, Atom, JSON Feed transparently), and keeps only entries whose `published` falls inside the last 24 hours.
3. `Paperboy::deliver` renders the `Vec<Feed>` through Handlebars (HTML template mandatory, text alternative optional) and passes it to `Mailer::send`, which uses `lettre` over SMTP (STARTTLS on by default; set `SMTP_STARTTLS=false` for local dev).

## Subscriptions file format

Plain text. One feed URL per line. A `#` at the start of a line comments it out. Leading/trailing whitespace around the URL is trimmed. Whitespace before `#` defeats the comment check — documented quirk, not a bug.

## Rules to keep in mind

- **No `native-tls` / `openssl`.** `reqwest` and `lettre` are both pinned to `rustls-tls`; adding any dep that pulls in `native-tls` will break the `x86_64-unknown-linux-musl` release build. Verify with `cargo tree -i native-tls` (should say "did not match any packages").
- **Don't panic on feed data.** `feed_rs::model::Entry` can legitimately have `title: None` or `links: vec![]`, and some feeds lack a channel title. The code uses `filter_map` / `unwrap_or_else` to skip/fallback; please preserve that discipline.
- **Crate versions are independent.** Bump `paperboy-cli` for binary changes; bump `paperboy` only when the library's public API changes.
- **Tags are the release source of truth.** `gh release create vX.Y.Z --generate-notes` fires `.github/workflows/release.yml` using the workflow file **at the tag's commit** — so any workflow fix must be merged *before* tagging.

## Commands

```bash
cargo test --all                   # 20 tests; uses mockito, no real network
cargo clippy --all                 # kept clean
cargo build --release --bin paperboy
docker compose up -d               # Mailpit on localhost:1025 (UI: :8025)
source .env.example && cargo run -- deliver reader@localhost subs_example.txt emails/daily_email.hbs emails/daily_email_text.hbs
```

## CI / Release

- `.github/workflows/ci.yml` runs on every PR and push to `main`. Jobs: `test` (cargo test on ubuntu) plus `release-targets` — cross-compiles for `x86_64-unknown-linux-musl`, `x86_64-apple-darwin`, and `aarch64-apple-darwin` so cross-compile regressions surface during review, not at release time.
- `.github/workflows/release.yml` triggers on `release: created` and builds the same three targets, uploading archives to the release. A `workflow_dispatch` input lets a failed release be retried against an existing tag without recreating it.
