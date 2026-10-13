<script lang="ts">
  import { onMount } from 'svelte'
  import { Trash2 } from 'lucide-svelte'
  import { SvelteMap } from 'svelte/reactivity'
  import { request } from './api'
  type Word = { word: string; translation: string | null; created_at: string }
  let words = $state<Word[]>([])
  const drafts = new SvelteMap<string, string>()
  let saving = $state<string[]>([])
  const rowErrors = new SvelteMap<string, string>()
  let word = $state('')
  let translation = $state('')
  let loading = $state(true)
  let busy = $state(false)
  let error = $state('')
  onMount(() => {
    void load()
  })

  async function load() {
    loading = true
    error = ''
    try {
      words = await request<Word[]>('vocabulary')
    } catch (cause) {
      error = String(cause)
    } finally {
      loading = false
    }
  }

  async function add(event: SubmitEvent) {
    event.preventDefault()
    if (busy || saving.length || !word.trim()) return
    busy = true
    error = ''
    try {
      await request('vocabulary', { word, translation })
      word = ''
      translation = ''
      await load()
    } catch (cause) {
      error = String(cause)
    } finally {
      busy = false
    }
  }

  async function save(item: Word) {
    if (saving.includes(item.word)) return
    const value = drafts.get(item.word) ?? item.translation ?? ''
    if (value === (item.translation ?? '')) return
    saving.push(item.word)
    rowErrors.delete(item.word)
    try {
      await request(
        'vocabulary',
        { word: item.word, translation: value },
        'PUT',
      )
      item.translation = value
      drafts.delete(item.word)
    } catch (cause) {
      rowErrors.set(item.word, String(cause))
    } finally {
      saving = saving.filter((name) => name !== item.word)
    }
  }

  async function remove(item: Word) {
    if (!confirm(`Are you sure you want to delete '${item.word}'?`)) return
    saving.push(item.word)
    rowErrors.delete(item.word)
    try {
      await request(
        `vocabulary?${new URLSearchParams({ word: item.word })}`,
        undefined,
        'DELETE',
      )
      words = words.filter((value) => value.word !== item.word)
      drafts.delete(item.word)
    } catch (cause) {
      rowErrors.set(item.word, String(cause))
    } finally {
      saving = saving.filter((name) => name !== item.word)
    }
  }

  async function importWords() {
    if (busy) return
    busy = true
    error = ''
    try {
      await request('vocabulary/from-flashcards', undefined, 'POST')
      await load()
    } catch (cause) {
      error = String(cause)
    } finally {
      busy = false
    }
  }
</script>

<section aria-label="Vocabulary" aria-busy={loading}>
  <h1 class="mb-5 text-3xl font-bold">Vocabulary</h1>
  <form class="mb-4 flex flex-wrap gap-3" onsubmit={add}>
    <label
      >Word<input
        class="block rounded border bg-white p-2"
        required
        maxlength="256"
        bind:value={word}
        disabled={busy}
      /></label
    >
    <label
      >Translation<input
        class="block rounded border bg-white p-2"
        maxlength="4096"
        bind:value={translation}
        disabled={busy}
      /></label
    >
    <button
      class="self-end rounded bg-accent px-4 py-2 text-white disabled:opacity-50"
      disabled={busy || saving.length > 0 || !word.trim()}>Add word</button
    >
  </form>
  {#if error}<p role="alert">{error}</p>
    <button
      class="p-2 underline"
      onclick={load}
      disabled={busy || saving.length > 0}>Reload vocabulary</button
    >{/if}
  {#if loading}<p role="status">Loading vocabulary…</p>{:else}
    <div class="overflow-x-auto rounded-xl border bg-white">
      <table class="w-full text-left">
        <thead class="bg-gray-50"
          ><tr
            ><th scope="col" class="p-3">Word</th><th scope="col" class="p-3"
              >Translation</th
            ><th scope="col" class="p-3">Created at</th><th
              scope="col"
              class="p-3">Actions</th
            ></tr
          ></thead
        >
        <tbody
          >{#each words as item (item.word)}<tr class="border-t">
              <td class="p-3">{item.word}</td>
              <td class="p-3"
                ><input
                  class="rounded border p-2"
                  aria-label={`Translation for ${item.word}`}
                  value={drafts.get(item.word) ?? item.translation ?? ''}
                  disabled={busy || saving.includes(item.word)}
                  oninput={(event) =>
                    drafts.set(item.word, event.currentTarget.value)}
                  onchange={() => save(item)}
                />{#if rowErrors.has(item.word)}<p role="alert">
                    {rowErrors.get(item.word)}
                  </p>
                  {#if drafts.has(item.word)}<button
                      class="p-2 underline"
                      onclick={() => save(item)}
                      disabled={busy || saving.includes(item.word)}
                      >Retry save</button
                    >{/if}{/if}</td
              >
              <td class="whitespace-nowrap p-3"
                ><time datetime={item.created_at}
                  >{item.created_at
                    .replace('T', ' ')
                    .replace(/(?:Z|\+00:00)$/, ' UTC')}</time
                ></td
              >
              <td class="p-3"
                ><button
                  class="inline-flex size-10 items-center justify-center rounded text-danger hover:bg-red-50 disabled:opacity-50"
                  aria-label={`Delete word: ${item.word}`}
                  title="Delete word"
                  disabled={busy || saving.includes(item.word)}
                  onclick={() => remove(item)}
                  ><Trash2 size={18} aria-hidden="true" /></button
                ></td
              >
            </tr>{/each}</tbody
        >
      </table>
    </div>
    {#if !words.length}<p class="py-4">No words yet.</p>{/if}
  {/if}
  <div class="mt-4 flex flex-wrap items-center justify-between gap-3">
    <span>Total: {words.length}</span><button
      class="rounded bg-accent px-4 py-2 text-white disabled:opacity-50"
      disabled={busy || loading || saving.length > 0}
      onclick={importWords}>{busy ? 'Working…' : 'Add from flashcards'}</button
    >
  </div>
</section>
