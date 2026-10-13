<script lang="ts">
  import { onDestroy } from 'svelte'

  const params = new URLSearchParams(window.location.search)
  let submitting = $state(false)
  let error = $state<string | null>(null)
  let saved = $state(false)
  let acknowledgementTimer: ReturnType<typeof setTimeout>

  onDestroy(() => clearTimeout(acknowledgementTimer))

  async function createCard(
    event: SubmitEvent & { currentTarget: HTMLFormElement },
  ) {
    event.preventDefault()
    if (submitting) return

    const form = event.currentTarget
    const data = new FormData(form)
    const text = (name: string) => data.get(name) as string
    const filename = (name: string) => (data.get(name) as File).name || null

    const request = {
      question: text('question'),
      answer: text('answer'),
      examples: text('examples') || null,
      source: text('source') || null,
      tags: [
        ...new Set(
          text('tags')
            .split(',')
            .map((tag) => tag.trim())
            .filter(Boolean),
        ),
      ],
      question_img_fname: filename('question_image'),
      answer_img_fname: filename('answer_image'),
    }

    submitting = true
    error = null
    saved = false
    clearTimeout(acknowledgementTimer)

    try {
      const response = await fetch('/api/cards', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      })
      if (!response.ok)
        throw new Error(`Failed to create card (HTTP ${response.status})`)

      form.reset()
      saved = true
      acknowledgementTimer = setTimeout(() => (saved = false), 3000)
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Failed to create card'
    } finally {
      submitting = false
    }
  }
</script>

<div class="mx-auto my-8 max-w-[600px] p-4">
  <div class="flex w-full max-w-md flex-col gap-4 rounded bg-white p-8 shadow">
    <form onsubmit={createCard} aria-busy={submitting}>
      <h1 class="mb-4 text-2xl font-bold">Add a new card</h1>
      <fieldset disabled={submitting}>
        <label>
          <span>Question*:</span>
          <input
            name="question"
            type="text"
            required
            value={params.get('question') ?? ''}
          />
        </label>
        <label class="ml-4">
          <span>Image:</span>
          <input
            name="question_image"
            type="file"
            accept="image/*"
            aria-label="Question image"
          />
        </label>
        <label>
          <span>Answer:</span>
          <textarea name="answer" rows="4" value={params.get('answer') ?? ''}
          ></textarea>
        </label>
        <label class="ml-4">
          <span>Image:</span>
          <input
            name="answer_image"
            type="file"
            accept="image/*"
            aria-label="Answer image"
          />
        </label>
        <label>
          <span>Examples:</span>
          <textarea name="examples" rows="4"></textarea>
        </label>
        <label>
          <span>Source:</span>
          <input name="source" type="text" value={params.get('source') ?? ''} />
        </label>
        <label>
          <span>Tags (comma separated):</span>
          <input name="tags" type="text" value={params.get('tag') ?? ''} />
        </label>
        <button
          class="mt-4 rounded bg-blue-500 px-6 py-2 text-white transition hover:bg-blue-600 disabled:opacity-50"
          type="submit"
          disabled={submitting}
        >
          {submitting ? 'Creating...' : 'Create Flashcard'}
        </button>
      </fieldset>
    </form>
    {#if saved}
      <p role="status" class="mt-2 font-semibold text-green-600">
        Card added successfully!
      </p>
    {/if}
    {#if error}
      <p role="alert" class="mt-2 text-red-600">{error}</p>
    {/if}
  </div>
</div>

<style>
  label {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  input,
  textarea {
    min-width: 0;
    border: 1px solid currentColor;
    border-radius: 0.25rem;
    padding: 0.5rem 0.75rem;
  }
</style>
