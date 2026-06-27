# About

leptos server functions are being replaced by Axum endpoints.
This will give us a formal HTTP API for re-use in other applications, e.g. LLM tool calls.

Already done:
- `GET  /api/cards/{id}` — `get_card` in `src/components/edit_card.rs`
- `POST /api/cards`      — `submit_card` in `src/components/add_card.rs`

---

## `GET /api/cards`

**Server function:** `get_all_cards` in `src/components/list_cards.rs`

**Query params:** `tag: Option<String>` — filter by tag when present.

**Response:** `Vec<Flashcard>`

**Used by:** `ListCards` component.

---

## `PUT /api/cards/{id}`

**Server function:** `update_card` in `src/components/edit_card.rs`

**Request body:**
```json
{
  "question": "string",
  "answer": "string",
  "examples": "string | null",
  "source": "string | null",
  "tags": ["string"],
  "question_img_fname": "string | null",
  "answer_img_fname": "string | null"
}
```

**Response:** `204 No Content`

**Used by:** `EditCard` component (`ActionForm` / `ServerAction`).

---

## `GET /api/cards/review`

**Server function:** `GetNextCards` in `src/components/review_cards.rs`

**Response:** `Vec<Flashcard>` — cards whose `review_after_secs` has elapsed since `last_reviewed`.

**Used by:** `ReviewAllCards` component.

---

## `POST /api/cards/{id}/review`

**Server function:** `submit_answer` in `src/components/review_cards.rs`

**Request body:**
```json
{ "remembered": true }
```

**Behaviour:**
- `remembered: true` → doubles `review_after_secs`
- `remembered: false` → resets interval to 6 hours

**Response:** `204 No Content`

**Used by:** `ReviewAllCards` and `ReviewByTag` components.

---

## `GET /api/cards/review/{tag}`

**Server function:** `get_cards_by_tag` in `src/components/review_by_tag.rs`

**Response:** `Vec<Flashcard>` — cards with the given tag whose review is due.

**Used by:** `ReviewByTag` component.

> Note: register this route before `GET /api/cards/{id}` so the literal segment `review` is matched first.
