<img src="https://raw.githubusercontent.com/eduardostuart/paperboy/main/.github/resources/paperboy.png" alt="Paperboy" width="160" align="right">

# Paperboy

[![ci](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/eduardostuart/paperboy/actions/workflows/ci.yml)

Paperboy is a [GitHub template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template) that delivers new posts from all your favorite sites by email. 

## Why?

Free, no trackers, easy to set up, and does the job.

## Usage

1. Click on `use this template`;
2. Update the `subscriptions.txt` file. Include your list of websites;
3. Include these environment variables (`GitHub Secrets`):

```bash
# Which service are you using? fastmail? gmail? sendgrid? ... 
SMTP_RELAY="smtp.fastmail.com" 

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

Bonus:

1. **If you want a different template:**
> Create a new `Handlebars` file in the `emails` folder or update the existing one.

2. **If you want to receive emails multiple times per day or at a different hour of the day**
> Update the `cron` property inside the workflow `deliver.yml`

## Tests

```
cargo run tests
```

## License

This code is distributed under the terms of MIT license.
See [LICENSE-MIT](LICENSE-MIT) for details.