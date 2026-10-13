<script lang="ts">
  import { onDestroy } from 'svelte'
  import ImageInput from './ImageInput.svelte'

  const params = new URLSearchParams(window.location.search)
  let submitting = $state(false)
  let error = $state<string | null>(null)
  let saved = $state(false)
  let acknowledgementTimer: ReturnType<typeof setTimeout>
  let questionImage = $state<File | null>(null)
  let answerImage = $state<File | null>(null)
  let questionImageInput: ReturnType<typeof ImageInput>
  let answerImageInput: ReturnType<typeof ImageInput>

  onDestroy(() => clearTimeout(acknowledgementTimer))

  async function createCard(
    event: SubmitEvent & { currentTarget: HTMLFormElement },
  ) {
    event.preventDefault()
    if (submitting) return

    const form = event.currentTarget
    const data = new FormData(form)
    const text = (name: string) => data.get(name) as string

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
    }

    submitting = true
    error = null
    saved = false
    clearTimeout(acknowledgementTimer)

    try {
      // ponytail: failed card creation can leave unused uploads; add cleanup when media management is needed.
      const question_img_fname = await uploadImage(questionImage)
      const answer_img_fname = await uploadImage(answerImage)
      const response = await fetch('/api/cards', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          ...request,
          question_img_fname,
          answer_img_fname,
        }),
      })
      if (!response.ok)
        throw new Error(`Failed to create card (HTTP ${response.status})`)

      form.reset()
      questionImage = null
      answerImage = null
      saved = true
      acknowledgementTimer = setTimeout(() => (saved = false), 3000)
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Failed to create card'
    } finally {
      submitting = false
    }
  }

  async function uploadImage(file: File | null): Promise<string | null> {
    if (!file) return null
    const data = new FormData()
    data.append('image', file)
    const response = await fetch('/api/media', { method: 'POST', body: data })
    if (!response.ok) {
      throw new Error(
        `Failed to upload image: ${(await response.text()) || response.status}`,
      )
    }
    const result: { filename: string } = await response.json()
    return result.filename
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
            onpaste={(event) => questionImageInput.paste(event)}
          />
        </label>
        <ImageInput
          label="Question image"
          bind:this={questionImageInput}
          bind:file={questionImage}
          disabled={submitting}
        />
        <label>
          <span>Answer:</span>
          <textarea
            name="answer"
            rows="4"
            value={params.get('answer') ?? ''}
            onpaste={(event) => answerImageInput.paste(event)}></textarea>
        </label>
        <ImageInput
          label="Answer image"
          bind:this={answerImageInput}
          bind:file={answerImage}
          disabled={submitting}
        />
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
