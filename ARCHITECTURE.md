# ESnip Data Architecture

## Goal

ESnip should remain responsive with very large local collections, including
hundreds of thousands to millions of text cards.

The core rule is simple: never load or render the full collection in the
frontend.

## Storage

Use SQLite as the local source of truth.

Current tables:

- `notes`: canonical note/card records.
- `notes_fts`: FTS5 virtual table indexing searchable text.
- `note_tags`: normalized tag index used for filtering and tag suggestions.

The main note table should store metadata needed for list rendering:

- `id`
- `title`
- `content`
- `kind`
- `created_at`
- `updated_at`
- `source`
- `tags_json`
- `tone`

Use normal indexes for common list filters and sorting:

- `(created_at DESC, id DESC)` on `notes`
- `(updated_at DESC, id DESC)` on `notes`
- `kind` on `notes`
- `title COLLATE NOCASE` on `notes`
- `tag COLLATE NOCASE` on `note_tags`
- `note_id` on `note_tags`

Use FTS5 for full-text search over `title` and `content`.

### Tag Storage

Tags are intentionally stored in two forms:

- `notes.tags_json`: display source of truth. It preserves user-facing order and
  casing and is loaded with the note row for frontend rendering.
- `note_tags`: normalized index. It supports efficient `EXISTS` filters,
  exclusion filters, and tag suggestion counts.

This is a deliberate denormalized design: `tags_json` is not redundant while the
UI needs ordered display tags, and `note_tags` is required for scalable tag
queries.

## Query Strategy

All list and search APIs must be paginated. The current implementation uses
keyset cursor pagination, not `OFFSET`.

Default page sizes:

- Home/recent list: `80` records, clamped to `1-100`.
- Search results: `80` records, clamped to `1-100`.

Current cursor shape:

- `updatedAt`: last row's `updated_at`.
- `id`: last row's stable tie-breaker.
- `rank`: optional FTS rank from `bm25(notes_fts)`.

Recent and filter-only pages use `(updated_at DESC, id DESC)`.

FTS pages use `(bm25(notes_fts), updated_at DESC, id DESC)`. The next page
condition is rank first, then `updated_at`, then `id`, so tied ranks still page
deterministically.

Count convention:

- First page returns the real `total_count`.
- Cursor pages return `total_count = -1` as a sentinel meaning "unchanged; keep
  the count already cached by the frontend".

Search APIs should return list-preview fields first, not full heavyweight
payloads. Load full content only when opening a card if the list preview is not
enough.

## Search

Use SQLite FTS5 for local full-text search.

Search constraints:

- Debounce user input before querying.
- Enforce a minimum query length where appropriate.
- Always apply `LIMIT`.
- Return snippets or preview text for result lists.
- Avoid sending thousands of full records across the Tauri IPC boundary.

Current short-query behavior:

- Queries with only filters (`#tag`, `!#tag`, `@kind`, `!@kind`) are allowed.
- Text-only queries require every text term to be at least 3 characters or at
  least 2 CJK characters.
- Terms that are allowed but too short for trigram FTS use the SQLite `LIKE`
  fallback.

Reason: SQLite FTS5 trigram search needs enough input to form trigram tokens.
Two CJK characters provide enough UTF-8 bytes for useful matching, while a
single CJK character is too broad and would fall back to a costly scan.

Search result ranking should be computed in SQLite where possible, then returned
as a small page of results.

## Frontend Rendering

The frontend must not load the full collection at once.

Current behavior:

- Initial render loads the first page: `80` cards.
- The notes area listens for scroll and requests the next page near the bottom
  of the scroll region.
- Loaded pages are appended to `notes.value`; the active DOM currently grows
  with browsing.
- Masonry columns are computed from the loaded notes only, not the full
  database.

Planned optimization:

- For very long browsing sessions, add virtualized masonry rendering or
  offscreen DOM recycling.
- Keep the normal active DOM window bounded, for example `200-500` cards.

The UI should never map over a collection containing tens of thousands of notes.

## Tauri Boundary

Keep data crossing the Tauri IPC boundary small.

Commands should return:

- A page of cards.
- A cursor for the next page.
- Lightweight list-preview fields.

Avoid returning:

- Entire database dumps.
- Large unbounded arrays.
- Full content for every search result.

## Performance Rules

Current hard rules and implementation constraints:

- Do not load all notes into frontend memory.
- Do not render all notes into the DOM.
- Do not use deep `OFFSET` pagination at million-record scale.
- Do not run full-text search in JavaScript over the full dataset.
- Do not return unbounded FTS results.
- Do not query on every keystroke without debounce.

SQLite + FTS5 + keyset cursor pagination is the current foundation for
million-scale local text collections. Bounded DOM rendering is still the main
remaining frontend scalability improvement.
