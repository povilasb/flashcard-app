<script lang="ts">
  import { onMount } from 'svelte'
  import { CirclePlus, Pencil } from 'lucide-svelte'
  import RichText from './RichText.svelte'

  let { tag, cardId }: { tag?: string; cardId?: string } = $props()

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
  let reviewingRandom = $state(false)
  let hasRandomCards = $state(true)
  let card = $derived(cards[index])

  onMount(() => {
    void loadCards()
  })

  async function loadCards(random = false) {
    loading = true
    error = null
    reviewingRandom = random

    try {
      const response = await fetch(
        cardId
          ? `/api/cards/${cardId}`
          : tag
            ? `/api/cards?${new URLSearchParams({ tag })}`
            : random
              ? '/api/cards/random'
              : '/api/cards/review',
      )
      if (!response.ok)
        throw new Error(`Failed to load cards (HTTP ${response.status})`)

      const result = await response.json()
      cards = cardId ? [result] : result
      index = 0
      showAnswer = false
      showExamples = false
      hasRandomCards = !random || cards.length > 0
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

<section
  class="min-w-0 text-ink wrap-anywhere [&_:is(button,a):focus-visible]:outline-2 [&_:is(button,a):focus-visible]:outline-offset-2 [&_:is(button,a):focus-visible]:outline-accent [&_.rich-text]:min-w-0 [&_.rich-text]:max-w-full [&_:is(img,svg,math)]:max-w-full [&_img]:h-auto [&_:is(pre,table)]:max-w-full [&_:is(pre,table)]:overflow-x-auto [&_table]:block"
  aria-label="Review cards"
  aria-busy={loading || submitting}
>
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
        <div class="mx-auto my-8 max-w-[600px] min-[760px]:p-4">
          <div
            class="mb-4 rounded-lg border border-line bg-surface p-4 text-left shadow sm:p-6"
          >
            <div class="flex flex-col items-start gap-4">
              <div class="w-full min-w-0 text-left text-[1.1rem] leading-6">
                <RichText text={card.question} />
              </div>
              {#if card.question_img}
                <img
                  src={mediaUrl(card.question_img)}
                  alt="Question illustration"
                  class="mt-4 h-auto max-w-full rounded-md border border-line"
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
                  class="mt-4 h-auto max-w-full rounded-md border border-line"
                />
              {/if}
            {/if}
            {#if showExamples}
              <div class="mt-4 rounded-md bg-chip p-4">
                <b>Examples:</b>
                <RichText text={card.examples ?? ''} />
              </div>
            {/if}
            <div class="mt-4 flex flex-wrap gap-2">
              {#each card.tags as tag}
                <span
                  class="max-w-full rounded-2xl border border-line bg-chip px-3 py-1 text-center text-sm text-subtle"
                  >{tag}</span
                >
              {/each}
            </div>
            <footer class="mt-4 flex flex-wrap items-center justify-end gap-2">
              <div
                class="min-w-0 flex-1 basis-36 text-right text-xs text-subtle"
              >
                Last reviewed: {reviewedAt(card.last_reviewed)}
              </div>
              <button
                type="button"
                aria-label="Show examples"
                title="Show examples"
                aria-expanded={showExamples}
                class="inline-flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-md text-subtle transition-colors hover:bg-chip hover:text-ink"
                onclick={() => (showExamples = true)}
                ><CirclePlus size={18} aria-hidden="true" /></button
              >
              <a
                href={`/cards/edit/${card.id}`}
                aria-label="Edit card"
                title="Edit card"
                class="inline-flex size-11 shrink-0 items-center justify-center rounded-md text-subtle transition-colors hover:bg-chip hover:text-ink"
                ><Pencil size={18} aria-hidden="true" /></a
              >
            </footer>
          </div>
          <div class="mt-4 flex justify-center gap-4">
            {#if showAnswer && !cardId}
              <button
                class="cursor-pointer rounded-md bg-accent-soft px-6 py-3 text-base text-accent transition-colors hover:bg-chip disabled:cursor-wait disabled:opacity-50"
                disabled={submitting}
                onclick={() => submitAnswer(true)}>Remember</button
              >
              <button
                class="cursor-pointer rounded-md bg-danger-soft px-6 py-3 text-base text-danger transition-colors hover:bg-chip disabled:cursor-wait disabled:opacity-50"
                disabled={submitting}
                onclick={() => submitAnswer(false)}>Not</button
              >
            {:else if !showAnswer}
              <button
                class="cursor-pointer rounded-md bg-accent-soft px-6 py-3 text-base text-accent transition-colors hover:bg-chip"
                onclick={() => (showAnswer = true)}>Show Answer</button
              >
            {/if}
          </div>
        </div>
      {/key}
    {:else if !error}
      <div class="mx-auto my-8 max-w-[600px] p-4 text-center">
        <p>Nothing to review at the moment.</p>
        {#if !tag && !cardId && hasRandomCards}
          <button
            class="mt-4 min-h-11 cursor-pointer rounded-md bg-accent-soft px-6 py-3 text-base text-accent transition-colors hover:bg-chip"
            onclick={() => loadCards(true)}>5 more</button
          >
        {/if}
      </div>
    {/if}
  {/if}
  {#if error}
    <p role="alert" class="mt-4 text-danger">{error}</p>
    {#if !card}<button
        class="mt-2 rounded bg-chip px-4 py-2 hover:bg-line"
        onclick={() => loadCards(reviewingRandom)}>Retry</button
      >{/if}
  {/if}
</section>
