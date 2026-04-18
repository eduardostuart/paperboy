<img src="https://raw.githubusercontent.com/eduardostuart/paperboy/main/.github/resources/paperboy.png" alt="Paperboy" width="210" align="right">

# Paperboy

[![ci](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml)

A CLI that mails you every fresh post from your RSS, Atom, and JSON feed subscriptions.

- Fetches feeds concurrently and keeps only entries published in the last 24 hours
- Renders the digest through a Handlebars HTML template (plus an optional plain-text alternative)
- Delivers through any SMTP relay and pairs with the [paperboy-template](https://github.com/eduardostuart/paperboy-template) GitHub Action for fully automated daily delivery

[`download`](https://github.com/eduardostuart/paperboy/releases/latest) · [`development & usage`](./docs/DEVELOPMENT.md) · [`license`](./LICENSE-MIT)

<br clear="both" />

## Author

[Eduardo Stuart](https://s.tuart.dev)
