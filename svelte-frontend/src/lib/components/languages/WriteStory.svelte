<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { on } from 'svelte/events'
  import { request, flashcardLink } from './api'
  let story = $state('')
  let language = $state('')
  let pending = $state(false)
  let error = $state('')
  let hovered = $state('')
  let translation = $state('')
  let hoverError = $state('')
  let hoverPending = $state(false)
  let x = $state(0)
  let y = $state(0)
  let selected = $state('')
  let selectedTranslation = $state('')
  let selectionPending = $state(false)
  let selectionError = $state('')
  let hoverRequest = 0
  let selectionRequest = 0
  let alive = true
  let content: HTMLDivElement

  onMount(() => {
    void generate()
    const pointer = on(content, 'pointerup', readSelection)
    const keyboard = on(content, 'keyup', readSelection)
    return () => {
      pointer()
      keyboard()
    }
  })
  onDestroy(() => {
    alive = false
    hoverRequest++
    selectionRequest++
  })

  async function generate() {
    if (pending) return
    pending = true
    error = ''
    try {
      const result = await request<{ story: string; language: string }>(
        'story',
        undefined,
        'POST',
      )
      if (alive) {
        story = result.story
        language = result.language
      }
    } catch (cause) {
      if (alive) error = String(cause)
    } finally {
      if (alive) pending = false
    }
  }

  async function showWord(raw: string, element: HTMLElement) {
    const word = raw.replace(/^[^\p{L}]+|[^\p{L}]+$/gu, '').toLowerCase()
    if (word && word === hovered && !hoverError) return
    clearHover()
    if (!word) return
    const id = hoverRequest
    const rect = element.getBoundingClientRect()
    x = Math.max(
      12,
      Math.min(rect.left, innerWidth - Math.min(280, innerWidth - 24) - 12),
    )
    y = Math.max(12, Math.min(rect.bottom + 6, innerHeight - 110))
    hovered = word
    hoverPending = true
    try {
      const lookup = await request<{ translation: string | null }>(
        `translation?${new URLSearchParams({ word })}`,
      )
      if (!alive || id !== hoverRequest) return
      const result = lookup.translation?.trim()
        ? lookup
        : await request<{ translation: string }>('translate', { text: word })
      if (alive && id === hoverRequest) translation = result.translation ?? ''
    } catch (cause) {
      if (alive && id === hoverRequest) hoverError = String(cause)
    } finally {
      if (alive && id === hoverRequest) hoverPending = false
    }
  }

  function clearHover() {
    hoverRequest++
    hovered = ''
    translation = ''
    hoverError = ''
    hoverPending = false
  }

  function readSelection() {
    const selection = window.getSelection()
    if (
      !selection ||
      !content.contains(selection.anchorNode) ||
      !content.contains(selection.focusNode)
    )
      return
    const text = selection.toString().trim()
    if (text === selected) return
    selected = text
    clearHover()
    void translateSelection()
  }

  async function translateSelection() {
    const id = ++selectionRequest
    selectedTranslation = ''
    selectionError = ''
    selectionPending = !!selected
    if (!selected) return
    try {
      const result = await request<{ translation: string }>('translate', {
        text: selected,
      })
      if (alive && id === selectionRequest)
        selectedTranslation = result.translation
    } catch (cause) {
      if (alive && id === selectionRequest) selectionError = String(cause)
    } finally {
      if (alive && id === selectionRequest) selectionPending = false
    }
  }

  function clearSelection() {
    selectionRequest++
    selected = ''
    selectedTranslation = ''
    selectionError = ''
    selectionPending = false
    window.getSelection()?.removeAllRanges()
    clearHover()
  }
</script>

<section aria-busy={pending}>
  <h1 class="mb-5 text-3xl font-bold">Story of the day</h1>
  {#if pending}<p role="status">Writing story…</p>{/if}
  {#if error}<p role="alert">{error}</p>
    <button class="my-3 rounded border p-3" onclick={generate}
      >Retry story</button
    >{/if}
  <div class="story" bind:this={content} aria-label="Story text">
    {#each story.split('\n') as paragraph}<p>
        {#each paragraph.split(/(\s+)/) as word}{#if word.trim()}<span
              role="button"
              tabindex="0"
              aria-label={`Translate ${word}`}
              onmouseenter={(event) => {
                if (!window.getSelection()?.toString())
                  void showWord(word, event.currentTarget)
              }}
              onmouseleave={clearHover}
              onfocus={(event) => showWord(word, event.currentTarget)}
              onblur={clearHover}
              onclick={(event) => {
                if (!window.getSelection()?.toString())
                  void showWord(word, event.currentTarget)
              }}
              onkeydown={(event) => {
                if (event.key === 'Enter' || event.key === ' ') {
                  event.preventDefault()
                  void showWord(word, event.currentTarget)
                }
                if (event.key === 'Escape') clearHover()
              }}>{word}</span
            >{:else}{word}{/if}{/each}
      </p>{/each}
  </div>
  {#if hovered}<div
      class="tooltip"
      role="status"
      style:left={`${x}px`}
      style:top={`${y}px`}
    >
      {hoverPending ? 'Translating…' : hoverError || translation}
    </div>{/if}
  {#if selected}
    <section
      class="mt-5 rounded-xl border bg-surface p-4"
      aria-label="Selected text"
      aria-busy={selectionPending}
    >
      <div class="mb-3 flex items-center justify-between gap-3">
        <h2 class="font-semibold">Selected</h2>
        <button class="rounded border px-3 py-2" onclick={clearSelection}
          >Clear selection</button
        >
      </div>
      <p>{selected}</p>
      <h2 class="mt-4 font-semibold">Translation</h2>
      {#if selectionPending}<p role="status">
          Translating selection…
        </p>{:else if selectionError}<p role="alert">{selectionError}</p>
        <button class="p-2 underline" onclick={translateSelection}
          >Retry translation</button
        >{:else}<p>{selectedTranslation}</p>{/if}
      {#if selectedTranslation && !selectionPending}<a
          class="mt-4 inline-block rounded bg-accent px-4 py-2 text-on-accent"
          href={flashcardLink(selected, language, selectedTranslation)}
          >Create flashcard</a
        >{/if}
    </section>
  {/if}
</section>

<style>
  .story {
    max-width: 56rem;
    margin: 0 auto;
    font-size: 18px;
    line-height: 1.8;
  }
  .story p {
    white-space: pre-wrap;
    min-height: 1em;
    overflow-wrap: anywhere;
  }
  .story span {
    cursor: pointer;
    border-radius: 4px;
    user-select: text;
  }
  .story span:hover,
  .story span:focus-visible {
    background: var(--color-chip);
  }
  .story span:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }
  .tooltip {
    position: fixed;
    z-index: 20;
    width: max-content;
    max-width: min(280px, calc(100vw - 24px));
    max-height: 96px;
    overflow: auto;
    padding: 8px 12px;
    border-radius: 8px;
    background: var(--color-toast);
    color: white;
    font-size: 14px;
    overflow-wrap: anywhere;
    pointer-events: none;
  }
</style>
