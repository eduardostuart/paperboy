# Development

## Configuration

Paperboy reads its SMTP settings from environment variables:

```bash
SMTP_HOST="smtp.mailtrap.io"           # SMTP host
SMTP_PORT=2525                         # SMTP port
SMTP_USERNAME="username"               # SMTP username
SMTP_PASSWORD="password"               # SMTP password
SMTP_FROM="Paperboy <rss@domain.com>"  # sender
SMTP_STARTTLS="true"                   # STARTTLS (default: true)
EMAIL_SUBJECT="RSS Daily"              # email subject (optional)
```

See `.env.example` for a set of values that work with the local Mailpit server.

## Usage

```
paperboy deliver <to> <subscription_file> <template_html> [template_text]
```

Example:

```bash
paperboy deliver reader@example.com subs_example.txt emails/daily_email.hbs emails/daily_email_text.hbs
```

Templates are rendered with [Handlebars](https://handlebarsjs.com/) and receive an `items` array (each element is a `Feed` with nested `entries`).

## Local development

Spin up [Mailpit](https://github.com/axllent/mailpit) for local email capture:

```bash
docker compose up -d
source .env.example
cargo run -- deliver reader@localhost subs_example.txt emails/daily_email.hbs emails/daily_email_text.hbs
```

Open <http://localhost:8025> to inspect the captured messages.

## Testing

```bash
cargo test --all
cargo clippy --all
```

## Project structure

- `crates/paperboy/` — library: feed loading, Handlebars rendering, SMTP delivery
- `crates/paperboy-cli/` — CLI wrapper and env-var parsing
- `emails/` — example Handlebars templates
- `docker-compose.yml` — Mailpit for local email testing
