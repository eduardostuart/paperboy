<img src="https://raw.githubusercontent.com/eduardostuart/paperboy/main/.github/resources/paperboy.png" alt="Paperboy" width="160" align="right">

# Paperboy

[![ci](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml)

Paperboy is a CLI tool that delivers new posts from your favorite sites by email.

## Usage

Set these environment variable values:

```bash
# Which service are you using? fastmail? gmail? sendgrid? ...
SMTP_HOST="smtp.fastmail.com"

# Credentials
SMTP_USERNAME="username"
SMTP_PASSWORD="password"

# Who will send new posts by email?
# Use "Name <email@domain.com>" format
SMTP_FROM="Paperboy <rss@your-domain.com>"

# The recipient
# Use "Name <email@domain.com>" format
MAIL_TO="Eduardo <to@your-domain.com>"
```

Deliver command:

```
paperboy deliver email@domain --verbose

# If you want to use a custom email template:
# paperboy deliver email@domain custom-template.hbs --verbose
# ^ example/default: ./emails/daily_email.hbs
```

## Tests

```
cargo run tests
```

## License

This code is distributed under the terms of MIT license.
See [LICENSE-MIT](LICENSE-MIT) for details.
