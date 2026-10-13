<script lang="ts">
  import { onMount } from 'svelte'
  import { CirclePlus, Pencil } from 'lucide-svelte'
  import RichText from './RichText.svelte'

  type ReviewCard = {
    id: number
    question: string
    answer: string
    examples: string | null
    img: string | null
    question_img: string | null
    tags: string[]
    last_reviewed: string
  }

  let cards = $state<ReviewCard[]>([])
  let index = $state(0)
  let loading = $state(true)
  let submitting = $state(false)
  let error = $state<string | null>(null)
  let showAnswer = $state(false)
  let showExamples = $state(false)
  let card = $derived(cards[index])

  onMount(() => {
    void loadCards()
  })

  async function loadCards() {
    loading = true
    error = null
    try {
      const response = await fetch('/api/cards/review')
      if (!response.ok)
        throw new Error(`Failed to load cards (HTTP ${response.status})`)
      cards = await response.json()
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Failed to load cards'
    } finally {
      loading = false
    }
  }

  async function submitAnswer(remembered: boolean) {
    if (!card || submitting) return
    submitting = true
    error = null
    try {
      const response = await fetch(`/api/cards/${card.id}/review`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ remembered }),
      })
      if (!response.ok)
        throw new Error(`Failed to submit answer (HTTP ${response.status})`)
      index += 1
      showAnswer = false
      showExamples = false
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Failed to submit answer'
    } finally {
      submitting = false
    }
  }

  function mediaUrl(filename: string) {
    return `/media/${filename.split('/').map(encodeURIComponent).join('/')}`
  }

  function reviewedAt(value: string) {
    const date = new Date(value)
    return Number.isNaN(date.getTime())
      ? 'Unknown'
      : date.toISOString().slice(0, 16).replace('T', ' ')
  }
</script>

<section aria-label="Review cards" aria-busy={loading || submitting}>
  {#if loading}
    <p role="status">Loading cards...</p>
  {:else}
    <progress
      class="h-2.5 w-full rounded-full"
      aria-label="Review progress"
      value={cards.length ? Math.min(index + 1, cards.length) : 0}
      max={Math.max(cards.length, 1)}
    ></progress>
    {#if card}
      {#key card.id}
        <div class="mx-auto my-8 max-w-[600px] p-4">
          <div
            class="relative mb-4 rounded-lg border border-slate-200 bg-white p-6 pb-12 text-left shadow"
          >
            <div class="flex flex-col items-start gap-4">
              <div class="text-left text-[1.1rem] leading-6">
                <RichText text={card.question} />
              </div>
              {#if card.question_img}
                <img
                  src={mediaUrl(card.question_img)}
                  alt="Question illustration"
                  class="mt-4 h-auto max-w-full rounded-md border border-slate-200"
                />
              {/if}
            </div>
            {#if showAnswer}
              <hr class="my-2 mb-4" />
              <div class="text-left text-[1.1rem] leading-6">
                <RichText text={card.answer} />
              </div>
              {#if card.img}
                <img
                  src={mediaUrl(card.img)}
                  alt="Answer illustration"
                  class="mt-4 h-auto max-w-full rounded-md border border-slate-200"
                />
              {/if}
            {/if}
            {#if showExamples}
              <div class="mt-4 rounded-md bg-gray-50 p-4">
                <b>Examples:</b>
                <RichText text={card.examples ?? ''} />
              </div>
            {/if}
            <div>
              {#each card.tags as tag}
                <span
                  class="mt-4 inline-block rounded-full border border-slate-200 bg-slate-100 px-3 py-1 text-center text-sm text-slate-500"
                  >{tag}</span
                >
              {/each}
            </div>
            <div class="absolute right-20 bottom-4 text-xs text-slate-400">
              Last reviewed: {reviewedAt(card.last_reviewed)}
            </div>
            <button
              type="button"
              aria-label="Show examples"
              title="Show examples"
              class="absolute right-12 bottom-4 text-slate-400 transition-colors hover:text-slate-600"
              onclick={() => (showExamples = true)}
              ><CirclePlus size={16} /></button
            >
            <a
              href={`/cards/edit/${card.id}`}
              aria-label="Edit card"
              title="Edit card"
              class="absolute right-4 bottom-4 text-slate-400 transition-colors hover:text-slate-600"
              ><Pencil size={16} /></a
            >
          </div>
          <div class="mt-4 flex justify-center gap-4">
            {#if showAnswer}
              <button
                class="cursor-pointer rounded-md bg-blue-100 px-6 py-3 text-base text-blue-700 transition-colors hover:bg-blue-200 disabled:cursor-wait disabled:opacity-50"
                disabled={submitting}
                onclick={() => submitAnswer(true)}>Remember</button
              >
              <button
                class="cursor-pointer rounded-md bg-red-100 px-6 py-3 text-base text-red-600 transition-colors hover:bg-red-200 disabled:cursor-wait disabled:opacity-50"
                disabled={submitting}
                onclick={() => submitAnswer(false)}>Not</button
              >
            {:else}
              <button
                class="cursor-pointer rounded-md bg-blue-100 px-6 py-3 text-base text-blue-700 transition-colors hover:bg-blue-200"
                onclick={() => (showAnswer = true)}>Show Answer</button
              >
            {/if}
          </div>
        </div>
      {/key}
    {:else if !error}
      <p class="mx-auto my-8 max-w-[600px] p-4 text-center">
        Nothing to review at the moment.
      </p>
    {/if}
  {/if}
  {#if error}
    <p role="alert" class="mt-4 text-red-600">{error}</p>
    {#if !card}<button
        class="mt-2 rounded bg-gray-100 px-4 py-2 hover:bg-gray-200"
        onclick={loadCards}>Retry</button
      >{/if}
  {/if}
</section>
