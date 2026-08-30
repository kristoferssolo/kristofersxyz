# Backup and recovery runbook

How to copy the portfolio database safely, prove that a copy restores, put a
restored copy into service, and take Owner access back after a suspected
compromise.

The database holds the portfolio content, the Owner password hash, and every
active session. Treat any copy of it as a credential store: readable only by the
application user, never committed, never served, never baked into an image.

## Where the commands run

Locally, run the `just` recipes from the repository root. They use the
`DATABASE_URL` in `.env`, which points at `data/portfolio.db`.

On a deployed host, the same operations are subcommands of the application
binary, so no extra tooling has to be installed in the container:

```sh
docker compose exec app /app/kristofersxyz backup /app/data/backups/portfolio-20260830T101500Z.db
docker compose exec app /app/kristofersxyz verify-restore /app/data/portfolio-restored.db
docker compose exec -it app /app/kristofersxyz set-password <username>
```

The application is not deployed yet, so only the local half of this runbook has
been exercised. Rehearse the restore on the real host before depending on it.

## Create a backup

```sh
just db-backup data/backups/portfolio-$(date -u +%Y%m%dT%H%M%SZ).db
```

The backup is safe to take while the application is serving traffic, and the
destination must not already exist, so an earlier backup is never replaced.

Do not snapshot a live database with `cp`. SQLx opens SQLite in write-ahead log
mode, so committed data can still live in `portfolio.db-wal` while the main file
lags behind it. Copying the files one at a time can miss those commits or catch
a page mid-write, and the result can look fine until it is needed. `backup` runs
`VACUUM INTO`, which reads the source through a single transaction and writes one
self-contained file with the log content folded in.

A file copy is only acceptable when the application is stopped and
`portfolio.db`, `portfolio.db-wal`, and `portfolio.db-shm` are copied together.

## Validate a backup

```sh
just db-verify-restore data/backups/portfolio-20260830T101500Z.db
```

The recipe copies the backup into a temporary directory, runs SQLite's
`PRAGMA integrity_check` over the copy, and revokes the copy's sessions. The
backup file itself is left untouched, and the temporary copy is deleted
afterwards. A backup that has never been restored is a guess, so check the ones
you intend to rely on.

## Restore into place

Three rules hold for every step below:

- Each command names the exact file it touches. No wildcards, no shell variable
  you have not printed first, no default path.
- The active database is replaced only by moving a file that has already passed
  verification.
- The database being replaced is kept as a rollback copy until the restored one
  is confirmed.

1. Stop the application. `docker compose stop app` on a host, or stop the
   development server locally.

2. Pick a backup and validate it, as above.

3. Stage it next to the active database, on the same filesystem so the final
   move is atomic:

   ```sh
   cp -- data/backups/portfolio-20260830T101500Z.db data/portfolio-restored.db
   ```

4. Verify the staged file and revoke the sessions it carries:

   ```sh
   cargo run --quiet --no-default-features --features ssr -- verify-restore data/portfolio-restored.db
   ```

   Do not skip this. A backup still contains the sessions that were live when it
   was taken, and restoring them hands access back to whoever holds those
   cookies. The command reports the integrity check and the number of revoked
   sessions, and it records a `database_restored` security event.

5. Keep the current database as the rollback copy. Move its log sidecars with
   it, so SQLite can never pair the new database with the old log:

   ```sh
   mv -- data/portfolio.db data/portfolio-previous.db
   mv -- data/portfolio.db-wal data/portfolio-previous.db-wal  # only if it exists
   mv -- data/portfolio.db-shm data/portfolio-previous.db-shm  # only if it exists
   ```

6. Put the verified copy into service:

   ```sh
   mv -- data/portfolio-restored.db data/portfolio.db
   ```

7. Start the application. Migrations run at startup, so a backup taken on an
   older schema is brought forward without a separate step.

8. Check the public pages, then the Owner login. Keep `portfolio-previous.db`
   until you are satisfied. To roll back, stop the application and reverse steps
   5 and 6 by name.

## Restore Owner access

Rotate the Owner password after any restore and after any suspected compromise:

```sh
cargo run --quiet --no-default-features --features ssr -- set-password <username>
```

The command prompts twice without echo, enforces the Owner strength policy, and
writes the new hash, a new session version, and the deletion of every session
row in one transaction. The restore already revoked the sessions inside the
backup; this closes the window since then and replaces a password that may have
been exposed.

Log in once afterwards to confirm the new password works, and confirm that a
browser holding the old session cookie is sent back to the login page.

## Disable and re-enable `/admin`

The application has no kill switch. Owner access is closed at the trusted proxy
in front of it, which is the same place that terminates TLS and rate-limits
logins. Block these path prefixes there, and return 404:

- `/admin`
- `/login`
- `/api/` (the server functions `login`, `logout`, `admin_session`, and
  `save_*`)

Public pages render from server-side content and never call those endpoints, so
the portfolio stays up while Owner access is closed.

Bring the paths back only after the restore is verified, the password is
rotated, and the security events are reviewed. Then load `/login`, sign in, and
confirm an editor save works.

## Review security events

Authentication, session, request-rejection, password-change, restore, and
content-change events are written to standard output on the
`kristofersxyz::security` target:

```sh
docker compose logs app | grep kristofersxyz::security
```

Look for:

- `authentication_failed` bursts, and `login_throttled` alongside them
- `authentication_succeeded` following a run of failures
- `session_rejected` with reason `corrupt` or `unrecognized`
- `csrf_rejected` and `request_body_rejected`
- `portfolio_changed` edits you did not make
- `owner_password_changed` or `database_restored` you did not run

The events carry no passwords, hashes, cookies, session identifiers, or request
bodies, so they can be copied into incident notes as they are.

## Rotate secrets after a suspected compromise

The application's own secrets are the Owner password hash and the session rows,
both handled above. The rest live outside this repository and are rotated at
their provider, then redeployed and confirmed to have replaced the old value:

- host and platform access: SSH keys, the Dokploy login, and any API tokens
- registry or CI credentials that can publish a new image
- the environment injected into the container, which should be restored from a
  known-good source even where a value is not secret: `DATABASE_URL`,
  `PUBLIC_ORIGIN`, `DEPLOYMENT_MODE`

There is no password pepper today. If one is introduced, keep it outside SQLite
and outside every database backup, write down where it lives before an incident
asks, and rotate it only with a re-hash plan, because a new pepper invalidates
every stored hash and locks the Owner out.

## Keep copies out of Git and off the site

- `*.db` and its `-wal` and `-shm` sidecars are ignored, as is any `backups/`
  directory. Run `git status` before committing after recovery work.
- Keep backups in `data/backups/` or outside the repository. Never under
  `public/` or `target/site/`, where they would be served.
- `.dockerignore` excludes `data/`, so a database cannot end up in an image.
- Restrict backups to the application user, and encrypt off-host copies with
  keys kept somewhere other than the backup media.

## What is covered by tests

`src/db/backup.rs` holds a test that backs up a seeded temporary database,
restores the copy, and checks that the portfolio content survives, that the copy
passes its integrity check, and that the sessions the backup carried are gone
before the copy is treated as ready. A second test checks that a backup refuses
to write over an existing file.
