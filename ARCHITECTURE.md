# ESnip Data Architecture

## Goal

ESnip should remain responsive with very large local collections, including
hundreds of thousands to millions of text cards.

The core rule is simple: never load or render the full collection in the
frontend.

## Storage

Use SQLite as the local source of truth.

Recommended tables:

- `notes`: canonical note/card records.
- `notes_fts`: FTS5 virtual table indexing searchable text.
- Optional supporting tables later: `tags`, `note_tags`, `sources`.

The main note table should store metadata needed for list rendering:

- `id`
- `title`
- `content`
- `kind`
- `created_at`
- `updated_at`
- `source`

Use normal indexes for common list filters and sorting:

- `created_at DESC`
- `updated_at DESC`
- `kind`

Use FTS5 for full-text search over `title` and `content`.

## Query Strategy

All list and search APIs must be paginated.

Default page sizes:

- Home/recent list: `50-100` records.
- Search results: `30-50` records.

Avoid deep `OFFSET` pagination for large collections. Use cursor pagination
instead.

Recommended cursors:

- Recent list: `(created_at, id)`.
- Updated list: `(updated_at, id)`.
- Search: rank plus a stable tie-breaker such as `id`.

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

Search result ranking should be computed in SQLite where possible, then returned
as a small page of results.

## Frontend Rendering

The frontend must only render a small active window of cards.

Initial approach:

- Render only the first page on load.
- Fetch the next page as the user scrolls.
- Keep the active DOM count bounded when browsing many pages.

Suggested limits:

- Initial render: about `50` cards.
- Normal loaded window: about `200-500` cards.
- For longer browsing sessions, use virtualized list/masonry rendering and
  recycle offscreen DOM.

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

- Do not load all notes into frontend memory.
- Do not render all notes into the DOM.
- Do not use deep `OFFSET` pagination at million-record scale.
- Do not run full-text search in JavaScript over the full dataset.
- Do not return unbounded FTS results.
- Do not query on every keystroke without debounce.

With these constraints, SQLite + FTS5 + cursor pagination + bounded frontend
rendering is a reasonable architecture for million-scale local text collections.
