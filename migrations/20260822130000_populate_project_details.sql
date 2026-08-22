UPDATE project
SET
    title = 'guenther',
    description_markdown = '## What it solves

Guenther turns supported Instagram, TikTok, X, and YouTube Shorts links into media that plays inside the Telegram conversation. Public posts stay in the chat instead of sending everyone through a browser or login prompt.

## System shape

```text
Telegram update
    -> Guenther router
    -> private Cobalt sidecar
    -> Telegram media response
```

The Rust process classifies each URL, sends the download request to Cobalt, and builds the Telegram response. Optional modules handle F1 schedules, persistent bingo games, and reusable voice lines without making those dependencies mandatory for the media path.

## Engineering evidence

The bingo module stores concurrent chat-local games in SQLite through SQLx. Entry imports are transactional, and existing cards keep their original entry text after later edits.

The Compose deployment keeps Cobalt on a private network with no host port. An optional proxy applies only to Cobalt traffic, so it never receives the Telegram bot token.'
WHERE slug = 'guenther';

UPDATE project
SET
    title = 'traxor',
    description_markdown = '## What it solves

Torrent operations stay in the terminal.

## System

Traxor presents Transmission RPC state through a keyboard-driven terminal interface.'
WHERE slug = 'traxor';

UPDATE project
SET
    title = 'cipher-workshop',
    description_markdown = '## What it explores

Cipher implementations share one Rust workspace and support command-line and browser interfaces.'
WHERE slug = 'cipher-workshop';

DELETE FROM project_technology
WHERE project_id = (SELECT id FROM project WHERE slug = 'guenther');

INSERT INTO project_technology (project_id, sort_order, item)
SELECT id AS project_id, 1 AS sort_order, 'Rust' AS item
FROM project
WHERE slug = 'guenther'
UNION ALL
SELECT id AS project_id, 2 AS sort_order, 'teloxide' AS item
FROM project
WHERE slug = 'guenther'
UNION ALL
SELECT id AS project_id, 3 AS sort_order, 'Cobalt' AS item
FROM project
WHERE slug = 'guenther'
UNION ALL
SELECT id AS project_id, 4 AS sort_order, 'SQLx and SQLite' AS item
FROM project
WHERE slug = 'guenther'
UNION ALL
SELECT id AS project_id, 5 AS sort_order, 'Docker Compose' AS item
FROM project
WHERE slug = 'guenther';
