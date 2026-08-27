# kristofers.xyz

My portfolio, at [kristofers.xyz](https://kristofers.xyz/). The page is a modal
editor: a buffer list on the left, a content pane on the right, and a bottom row
that is the statusline until you open the command line.

The vim keys sit on top of an ordinary site. Every action has a visible
equivalent, and nothing is reachable only by keyboard.

## Stack

Leptos 0.8 with server-side rendering and hydration, Axum, Tailwind, SQLite via
sqlx. Built with [cargo-leptos](https://github.com/leptos-rs/cargo-leptos).

## Layout

```text
src/app/editor/   pure reducer over key input, no web_sys, no DOM
src/app/pages/    public Leptos route components
src/app/admin/    authenticated Leptos routes, forms, and server functions
src/app/content.rs  reactive portfolio content shared across routes
src/authentication/  credentials and typed owner-session transitions
src/db/           SQLite content and session persistence
migrations/       SQLite schema
end2end/          Playwright tests
```

The editor core is pure, so every key is unit tested without a browser. The
Leptos adapter normalizes the browser event into a `KeyInput`, calls `reduce`,
and applies the resulting effects.

## Development

```sh
just setup    # cargo-nextest and sccache
just run      # cargo leptos watch
just check    # fmt, clippy, sqruff, docs, test
just end2end  # Playwright
```

`DATABASE_URL` selects the SQLite database. `PUBLIC_ORIGIN` is the exact
scheme, host, and optional port accepted for state-changing browser requests,
for example `http://localhost:3000` during local development.
`DEPLOYMENT_MODE` must be one of:

- `local`, paired with an HTTP origin
- `production-behind-trusted-proxy`, paired with an HTTPS origin and a proxy
  that terminates TLS and replaces untrusted forwarding headers

Production mode always uses a Secure, host-only
`__Host-kristofersxyz-session` cookie. Startup rejects a deployment mode whose
transport does not match `PUBLIC_ORIGIN`. Startup then applies migrations and
loads the portfolio content before serving requests.

## License

MIT. See [LICENSE](LICENSE).
