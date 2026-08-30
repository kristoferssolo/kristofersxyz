# kristofers.xyz

My portfolio, at [kristofers.xyz](https://kristofers.xyz/). The page is a modal
editor: a buffer list on the left, a content pane on the right, and a bottom row
that is the statusline until you open the command line.

The vim keys sit on top of an ordinary site. Every action has a visible
equivalent, and nothing is reachable only by keyboard.

## Stack

Leptos 0.8 with server-side rendering and hydration, Axum, axum-login,
Tailwind, and SQLite via SQLx. Built with
[cargo-leptos](https://github.com/leptos-rs/cargo-leptos).

## Layout

```text
src/app/editor/   pure reducer over key input, no web_sys, no DOM
src/app/pages/    public Leptos route components
src/app/admin/    authenticated Leptos routes, forms, and server functions
src/app/content.rs  reactive portfolio content shared across routes
src/authentication/  axum-login backend and typed Owner-session policy
src/db/           SQLite content and session persistence
migrations/       SQLite schema
end2end/          Playwright tests
```

The editor core is pure, so every key is unit tested without a browser. The
Leptos adapter normalizes the browser event into a `KeyInput`, calls `reduce`,
and applies the resulting effects.

## Development

```sh
just setup    # cargo-nextest, sccache, and cargo-deny
just run      # cargo leptos watch
just check    # fmt, clippy, sqruff, docs, test
just security # cargo-deny: advisories, licenses, sources, bans
just end2end  # Playwright
just benchmark-auth  # serial and two-at-once Argon2 measurements
just db-backup data/backups/portfolio.db        # consistent copy of the database
just db-verify-restore data/backups/portfolio.db  # restore it to a temporary copy and check it
```

`DATABASE_URL` selects the SQLite database. `PUBLIC_ORIGIN` is the exact
scheme, host, and optional port accepted for state-changing browser requests,
for example `http://localhost:3000` during local development.
`DEPLOYMENT_MODE` must be one of:

- `local`, paired with an HTTP origin
- `production-behind-trusted-proxy`, paired with an HTTPS origin and a proxy
  that terminates TLS, blocks direct access to the application, and rate-limits
  login attempts by client address

Production mode always uses a Secure, host-only
`__Host-kristofersxyz-session` cookie. Startup rejects a deployment mode whose
transport does not match `PUBLIC_ORIGIN`. Startup then applies migrations and
loads the portfolio content before serving requests.

The application ignores forwarded client-address headers. Its in-process
source limit therefore sees the proxy as the peer; production needs the
per-client limit at that trusted edge.

Authentication, session, request-rejection, password-change, and content-change
events use the `kristofersxyz::security` tracing target. Informational events
form the audit trail. Warnings describe denied or sensitive operations, and
errors describe session lifecycle failures. The application writes them to
standard output without credentials, hashes, cookies, session identifiers,
request bodies, or raw authentication errors. Production log collection should
route warning and error events from this target to the Owner's alert channel.

`just security` runs `cargo deny check` against [deny.toml](deny.toml): RustSec
advisories, the accepted license list, crate sources, and version bans. It stays
out of `just check` because it fetches the advisory database over the network.
CI runs the same policy on pull requests, on pushes to `main`, and weekly, so an
advisory published after a merge still surfaces. When it reports a finding,
upgrade the dependency rather than adding an exception.

[docs/backup-recovery.md](docs/backup-recovery.md) is the operations runbook:
taking a consistent backup, checking that it restores, replacing the active
database while keeping a rollback copy, and taking Owner access back after a
suspected compromise.

Run `just benchmark-auth` on the smallest production host before changing the
Argon2 policy. Record both benchmark results with the host class in deployment
notes. The two-at-once case matches the application's maximum hashing
concurrency; do not lower the policy below its documented security baseline to
hit a timing target.

## License

MIT. See [LICENSE](LICENSE).
