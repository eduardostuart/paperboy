<img src="https://raw.githubusercontent.com/eduardostuart/paperboy/main/.github/resources/paperboy.png" alt="Paperboy" width="160" align="right">

# Paperboy

[![ci](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml)

Paperboy is a CLI tool that delivers new posts from your favorite sites by email.

## Installation

**From binaries**

Check out the [release page](https://github.com/eduardostuart/paperboy/releases/) for prebuilt versions of `Paperboy`.

## Usage

Set these environment variable values:

```bash
SMTP_HOST="smtp.fastmail.com" # Which service are you using? fastmail? gmail? sendgrid? ...
SMTP_PORT=465 # 25 is the default port
SMTP_USERNAME="username"
SMTP_PASSWORD="password"
SMTP_FROM="Paperboy <rss@your-domain.com>" # Who will send new posts by email?
```

Deliver command:

```bash
paperboy deliver email@domain subscriptions.txt template.hbs
# Template example.: emails/daily_email.hbs
```

## GitHub template

If you want an **automatic** and **free** way to receive daily "newspapers" using `paperboy`, take a look at the [paperboy-template](https://github.com/eduardostuart/paperboy-template) GitHub project.

## Tests

```
cargo test
```

## License

This code is distributed under the terms of MIT license.
See [LICENSE-MIT](LICENSE-MIT) for details.
