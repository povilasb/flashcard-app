<script lang="ts">
  // Browse cards and defer destructive requests until their undo window closes.
  import { onMount, tick } from 'svelte'
  import { Search, Plus, Pencil, Trash2, X } from 'lucide-svelte'
  import '@fontsource/figtree/400.css'
  import '@fontsource/figtree/500.css'
  import '@fontsource/figtree/600.css'
  import '@fontsource/figtree/700.css'

  type Card = {
    id: number
    question: string
    answer: string
    tags: string[]
    last_reviewed: string
  }
  type PendingDelete = { card: Card; timer: ReturnType<typeof setTimeout> }
  const STALE_DAYS = 180
  let cards = $state<Card[]>([])
  let loading = $state(true)
  let error = $state<string | null>(null)
  let deleteError = $state<string | null>(null)
  let search = $state('')
  let tag = $state<string | null>(null)
  let showAllTags = $state(false)
  let sort = $state('recent')
  let pending = $state<PendingDelete[]>([])
  let deleting = $state<number[]>([])
  let searchInput = $state<HTMLInputElement>()
  let section: HTMLElement
  const now = Date.now()
  let available = $derived(
    cards.filter(
      (card) =>
        !pending.some((item) => item.card.id === card.id) &&
        !deleting.includes(card.id),
    ),
  )
  let tagCounts = $derived.by(() => {
    const counts = new Map<string, number>()
    for (const card of available)
      for (const name of new Set(card.tags))
        counts.set(name, (counts.get(name) ?? 0) + 1)
    return [...counts].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
  })
  let visible = $derived(
    available
      .filter(
        (card) =>
          (!tag || card.tags.includes(tag)) &&
          [card.question, card.answer, ...card.tags].some((value) =>
            value.toLowerCase().includes(search.toLowerCase()),
          ),
      )
      .sort((a, b) =>
        sort === 'question'
          ? a.question.localeCompare(b.question)
          : (sort === 'oldest' ? 1 : -1) *
            (timestamp(a.last_reviewed) - timestamp(b.last_reviewed)),
      ),
  )
  let latest = $derived(pending.at(-1))

  onMount(() => {
    void loadCards()
    window.addEventListener('keydown', focusSearch)
    window.addEventListener('pagehide', flushDeletes)
    return () => {
      window.removeEventListener('keydown', focusSearch)
      window.removeEventListener('pagehide', flushDeletes)
      flushDeletes()
    }
  })

  async function loadCards() {
    loading = true
    error = null
    try {
      const response = await fetch('/api/cards')
      if (!response.ok)
        throw new Error(`Failed to load cards (HTTP ${response.status})`)
      cards = await response.json()
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Failed to load cards'
    } finally {
      loading = false
    }
  }

  function focusSearch(event: KeyboardEvent) {
    if (
      event.key !== '/' ||
      event.metaKey ||
      event.ctrlKey ||
      event.altKey ||
      event.isComposing
    )
      return
    const target = event.target
    if (
      target instanceof HTMLElement &&
      (target.isContentEditable || target.closest('input, textarea, select'))
    )
      return
    event.preventDefault()
    searchInput?.focus()
  }

  async function removeCard(card: Card) {
    const index = visible.findIndex((item) => item.id === card.id)
    const next = visible[index + 1] ?? visible[index - 1]
    deleteError = null
    pending.push({
      card,
      timer: setTimeout(() => void commitDelete(card.id), 5000),
    })
    await tick()
    const button = next
      ? section.querySelector<HTMLButtonElement>(
          `[data-delete-id="${next.id}"]`,
        )
      : null
    ;(button ?? searchInput)?.focus()
  }

  async function commitDelete(id: number) {
    const item = pending.find((item) => item.card.id === id)
    if (!item) return
    clearTimeout(item.timer)
    deleting.push(id)
    pending = pending.filter((item) => item.card.id !== id)
    try {
      // Keep in-flight deletes alive too if navigation follows the undo deadline.
      const response = await fetch(`/api/cards/${id}`, {
        method: 'DELETE',
        keepalive: true,
      })
      if (!response.ok)
        throw new Error(`Couldn't delete card. HTTP ${response.status}`)
      cards = cards.filter((card) => card.id !== id)
    } catch (cause) {
      deleteError =
        cause instanceof Error ? cause.message : "Couldn't delete card."
    } finally {
      deleting = deleting.filter((value) => value !== id)
    }
  }

  function undo() {
    const item = pending.at(-1)
    if (!item) return
    clearTimeout(item.timer)
    pending = pending.slice(0, -1)
  }

  function flushDeletes() {
    for (const item of [...pending]) void commitDelete(item.card.id)
  }

  function clearFilters() {
    search = ''
    tag = null
  }

  function timestamp(value: string) {
    return Date.parse(value) || 0
  }

  function age(value: string) {
    return Math.max(0, Math.floor((now - Date.parse(value)) / 86400000))
  }

  function relativeDate(value: string) {
    const days = age(value)
    if (Number.isNaN(days)) return 'Unknown'
    if (days === 0) return 'Today'
    if (days === 1) return 'Yesterday'
    const [count, unit] =
      days < 14
        ? [days, 'day']
        : days < 60
          ? [Math.floor(days / 7), 'week']
          : days < 365
            ? [Math.floor(days / 30), 'month']
            : [Math.floor(days / 365), 'year']
    return `${count} ${unit}${count === 1 ? '' : 's'} ago`
  }

  function formatReviewedAt(value: string) {
    const date = new Date(value)
    return Number.isNaN(date.getTime())
      ? 'Unknown'
      : date.toISOString().slice(0, 16).replace('T', ' ')
  }
</script>

<section
  class="page"
  aria-label="Cards"
  aria-busy={loading}
  bind:this={section}
>
  <div class="wrap">
    <header class="header">
      <div class="title">
        <h1>Cards</h1>
        <span class="count" aria-label="Card count">{available.length}</span>
      </div>
      <a class="primary" href="/add-card"
        ><Plus size={18} aria-hidden="true" />New card</a
      >
    </header>
    {#if loading}
      <p role="status">Loading cards…</p>
    {:else if error}
      <p role="alert">{error}</p>
      <button class="ghost" onclick={loadCards}>Retry</button>
    {:else}
      <div class="panel toolbar">
        <div class="toolbar-row">
          <div class="field search">
            <Search size={18} aria-hidden="true" /><input
              type="search"
              aria-label="Search cards"
              placeholder="Search questions, answers and tags"
              bind:this={searchInput}
              bind:value={search}
              onkeydown={(event) => {
                if (event.key === 'Escape') search = ''
              }}
            />{#if search}<button
                class="act"
                aria-label="Clear search"
                onclick={() => {
                  search = ''
                  searchInput?.focus()
                }}><X size={16} aria-hidden="true" /></button
              >{:else}<span class="kbd" aria-hidden="true">/</span>{/if}
          </div>
          <div class="sort">
            <label for="card-sort">Sort</label><select
              id="card-sort"
              class="field"
              bind:value={sort}
              ><option value="recent">Recently reviewed</option><option
                value="oldest">Least recently reviewed</option
              ><option value="question">Question A–Z</option></select
            >
          </div>
        </div>
        <div class="filters" role="group" aria-label="Filter by tag">
          <button
            class="filter"
            aria-pressed={tag === null}
            onclick={() => (tag = null)}
            >All <span>{available.length}</span></button
          >
          {#each showAllTags ? tagCounts : tagCounts.slice(0, 5) as [name, count]}<button
              class="filter"
              aria-pressed={tag === name}
              onclick={() => (tag = tag === name ? null : name)}
              >{name} <span>{count}</span></button
            >{/each}
          {#if tagCounts.length > 5}
            <button
              class="filter"
              aria-expanded={showAllTags}
              onclick={() => (showAllTags = !showAllTags)}
              >{showAllTags
                ? 'Show less'
                : `Show more (${tagCounts.length - 5})`}</button
            >
          {/if}
        </div>
      </div>
      <div class="panel table-panel">
        {#if visible.length}
          <!-- svelte-ignore a11y_no_noninteractive_tabindex (Keyboard users need to scroll the table.) -->
          <div
            class="scroll"
            tabindex="0"
            role="region"
            aria-label="Cards table"
          >
            <table>
              <thead
                ><tr
                  ><th scope="col">Question</th><th scope="col" class="answer"
                    >Answer</th
                  ><th scope="col">Tags</th><th scope="col">Reviewed</th><th
                    scope="col"><span class="sr-only">Actions</span></th
                  ></tr
                ></thead
              >
              <tbody
                >{#each visible as card (card.id)}<tr>
                    <td class="q"
                      ><a href={`/cards/edit/${card.id}`}
                        >{card.question || 'Untitled card'}</a
                      ></td
                    >
                    <td class="a">{card.answer}</td>
                    <td class="tags"
                      ><div>
                        {#each card.tags as name}<button
                            class="tag"
                            aria-label={`Filter by tag ${name}`}
                            onclick={() => (tag = name)}>{name}</button
                          >{/each}
                      </div></td
                    >
                    <td class="when"
                      ><time datetime={card.last_reviewed}
                        ><span
                          class="rel"
                          class:stale={age(card.last_reviewed) >= STALE_DAYS}
                          >{#if age(card.last_reviewed) >= STALE_DAYS}<span
                              class="dot"
                              aria-hidden="true"
                            ></span>{/if}{relativeDate(
                            card.last_reviewed,
                          )}</span
                        ><span class="abs"
                          >{formatReviewedAt(card.last_reviewed)}</span
                        ></time
                      ></td
                    >
                    <td class="actions"
                      ><div>
                        <a
                          class="act"
                          href={`/cards/edit/${card.id}`}
                          aria-label={`Edit card: ${card.question}`}
                          title="Edit card"
                          ><Pencil size={18} aria-hidden="true" /></a
                        ><button
                          class="act danger"
                          aria-label={`Delete card: ${card.question}`}
                          title="Delete card"
                          data-delete-id={card.id}
                          onclick={() => removeCard(card)}
                          ><Trash2 size={18} aria-hidden="true" /></button
                        >
                      </div></td
                    >
                  </tr>{/each}</tbody
              >
            </table>
          </div>
        {:else if !available.length}
          <div class="empty">
            <strong>No cards yet.</strong><a class="primary" href="/add-card"
              >New card</a
            >
          </div>
        {:else}
          <div class="empty">
            <strong>No cards match</strong>
            <p>Try a different search or tag.</p>
            <button class="ghost" onclick={clearFilters}>Clear filters</button>
          </div>
        {/if}
        <footer class="foot">
          <span
            >{search || tag
              ? `Showing ${visible.length} of ${available.length} cards`
              : `${available.length} cards`}</span
          ><span class="legend"
            ><span class="dot" aria-hidden="true"></span>Not reviewed in 6+
            months</span
          >
        </footer>
      </div>
    {/if}
  </div>
  <div class="notifications">
    {#if deleteError}<div class="snackbar" role="alert">
        <span>{deleteError}</span><button
          aria-label="Dismiss error"
          onclick={() => (deleteError = null)}
          ><X size={18} aria-hidden="true" /></button
        >
      </div>{/if}
    {#if latest}<div class="snackbar" role="status">
        <span class="msg"
          >Deleted "{latest.card.question.length > 40
            ? `${latest.card.question.slice(0, 40)}…`
            : latest.card.question}"</span
        ><button onclick={undo}>Undo</button>
      </div>{/if}
  </div>
</section>

<style>
  .page {
    min-height: 100%;
    padding: 40px 24px 56px;
    background: var(--color-page);
    color: var(--color-ink);
    font-family: 'Figtree', system-ui, sans-serif;
    line-height: normal;
  }
  .wrap {
    max-width: 1160px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  :focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }
  button {
    cursor: pointer;
  }
  .header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    padding-bottom: 8px;
  }
  .title {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }
  h1 {
    font-size: 32px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .count {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-subtle);
    background: var(--color-surface);
    border: 1px solid var(--color-line);
    border-radius: 999px;
    padding: 4px 12px;
  }
  .primary {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 44px;
    padding: 0 18px 0 14px;
    border-radius: 10px;
    background: var(--color-accent);
    color: var(--color-on-accent);
    font-size: 15px;
    font-weight: 600;
  }
  .primary:hover {
    filter: brightness(1.08);
  }
  .panel {
    background: var(--color-surface);
    border: 1px solid var(--color-line);
    border-radius: 16px;
  }
  .toolbar {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .toolbar-row {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
  }
  .field {
    border: 1px solid var(--color-field);
    border-radius: 10px;
    background: var(--color-surface);
  }
  .field:hover {
    border-color: var(--color-field-hover);
  }
  .field:focus-within {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px rgb(53 84 209 / 0.18);
  }
  .search {
    flex: 1 1 320px;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 6px 0 14px;
    color: var(--color-hint);
  }
  .search input {
    flex: 1;
    min-width: 0;
    height: 100%;
    outline: none;
    font-size: 15px;
    color: var(--color-ink);
  }
  .search input::placeholder {
    color: var(--color-placeholder);
  }
  .search input::-webkit-search-cancel-button {
    display: none;
  }
  .sort {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: var(--color-subtle);
    min-width: 0;
  }
  .sort select {
    height: 44px;
    min-width: 0;
    padding: 0 12px;
    font-size: 15px;
    font-weight: 500;
    color: var(--color-ink);
  }
  .kbd {
    font-size: 12px;
    font-weight: 600;
    border: 1px solid var(--color-line);
    border-radius: 6px;
    padding: 2px 6px;
  }
  .filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .filter {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 40px;
    max-width: 100%;
    overflow-wrap: anywhere;
    padding: 0 12px;
    border: 1px solid var(--color-line);
    border-radius: 999px;
    font-size: 14px;
    font-weight: 500;
  }
  .filter span {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-hint);
  }
  .filter[aria-pressed='true'] {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-on-accent);
  }
  .filter[aria-pressed='true'] span {
    color: var(--color-on-accent);
    opacity: 0.8;
  }
  .table-panel {
    overflow: hidden;
  }
  .scroll {
    position: relative;
    overflow-x: auto;
  }
  table {
    width: 100%;
    min-width: 760px;
    border-collapse: collapse;
    font-size: 15px;
  }
  th {
    text-align: left;
    padding: 12px 20px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-subtle);
    background: var(--color-table-head);
    border-bottom: 1px solid var(--color-divider);
  }
  th.answer {
    color: var(--color-back);
  }
  td {
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-divider);
    vertical-align: top;
    overflow-wrap: break-word;
  }
  tbody tr:hover {
    background: var(--color-surface-hover);
  }
  .q {
    font-weight: 600;
    line-height: 1.45;
    width: 32%;
  }
  .q a:hover {
    color: var(--color-accent);
    text-decoration: underline;
  }
  .a {
    color: var(--color-ink-2);
    line-height: 1.5;
  }
  .tags {
    padding-top: 14px;
    width: 170px;
  }
  .tags div {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .tag {
    min-height: 26px;
    padding: 0 10px;
    border: 1px solid transparent;
    border-radius: 999px;
    background: var(--color-chip);
    color: var(--color-chip-ink);
    font-size: 13px;
    font-weight: 500;
  }
  .tag:hover {
    border-color: var(--color-field-hover);
  }
  .when {
    width: 140px;
    white-space: nowrap;
  }
  time {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .rel {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-ink-2);
  }
  .rel.stale {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
    color: var(--color-back);
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: #d97a1e;
    flex: none;
  }
  .abs {
    font-size: 12px;
    color: var(--color-hint);
  }
  .actions {
    padding: 10px 12px;
    width: 100px;
  }
  .actions div {
    display: flex;
    justify-content: flex-end;
    gap: 4px;
  }
  .act {
    width: 40px;
    height: 40px;
    flex: none;
    border-radius: 10px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--color-subtle);
  }
  .act:hover {
    background: var(--color-chip);
    color: var(--color-ink);
  }
  .act.danger:hover {
    background: var(--color-danger-soft);
    color: var(--color-danger);
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 56px 24px;
    text-align: center;
  }
  .empty strong {
    font-size: 17px;
  }
  .empty p {
    font-size: 14px;
    color: var(--color-subtle);
  }
  .ghost {
    min-height: 44px;
    padding: 0 16px;
    border: 1px solid var(--color-field);
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
  }
  .ghost:hover {
    background: var(--color-chip);
  }
  .foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 20px;
    font-size: 13px;
    color: var(--color-subtle);
    background: var(--color-surface-hover);
  }
  .legend {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .notifications {
    position: fixed;
    z-index: 10;
    left: 50%;
    bottom: 24px;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: calc(100% - 32px);
  }
  .snackbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 6px 6px 16px;
    border-radius: 12px;
    background: var(--color-toast);
    color: white;
    font-size: 14px;
    font-weight: 500;
    box-shadow: 0 10px 30px rgb(21 23 28 / 0.25);
  }
  .msg {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .snackbar button {
    min-height: 40px;
    padding: 0 12px;
    border-radius: 8px;
    color: #b6c4ff;
    font-weight: 700;
    flex: none;
  }
</style>
