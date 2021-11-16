<img src="https://raw.githubusercontent.com/eduardostuart/paperboy/main/.github/resources/paperboy.jpg" alt="Paperboy" width="250" align="right">

# Paperboy

[![ci](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml)

## Usage

Paperboy is a [GitHub template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) which includes workflows to deliver RSS by email.

1. The first thing you'll need to do is click on `"use this template"`;
2. Update the `"subscriptions.txt"` file. Include your list of websites;
3. You'll also need to include some environment variables (`GitHub Secrets`):

```bash
# Which service are you using? fastmail? gmail? sendgrid? ... 
SMTP_RELAY="smtp.fastmail.com" 

# Credentials
SMTP_USERNAME="username" 
SMTP_PASSWORD="password" 

# Who is sending this email? Use "Name <email@domain.com>" format
SMTP_FROM="RSS <rss@your-domain.com>" 

# Who will receive this the daily RSS email?
MAIL_TO="Eduardo <to@domain.com>" 
```

## Tests

```
cargo run tests
```

## License

This code is distributed under the terms of MIT license.
See [LICENSE-MIT](LICENSE-MIT) for details.