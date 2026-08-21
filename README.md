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
src/app/pages/    the Leptos adapter that renders editor state
src/app/content.rs  the portfolio as static data
src/db/           connection pool, unused until content moves to SQLite
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

`DATABASE_URL` is optional. Without it the site boots and serves its static
content.

## License

MIT. See [LICENSE](LICENSE).
