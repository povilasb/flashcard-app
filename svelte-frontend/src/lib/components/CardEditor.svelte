<script module lang="ts">
  export type EditableCard = {
    id: number
    question: string
    answer: string
    examples: string | null
    source: string | null
    tags: string[]
    question_img: string | null
    img: string | null
  }
</script>

<script lang="ts">
  import { onDestroy, onMount, tick, untrack } from 'svelte'
  import { on } from 'svelte/events'
  import { Check, ChevronLeft, Link, Plus, X } from 'lucide-svelte'
  import '@fontsource/figtree/400.css'
  import '@fontsource/figtree/500.css'
  import '@fontsource/figtree/600.css'
  import '@fontsource/figtree/700.css'
  import '@fontsource/newsreader/400.css'
  import '@fontsource/newsreader/500.css'
  import '@fontsource/newsreader/400-italic.css'
  import ImageInput from './ImageInput.svelte'

  let { card }: { card?: EditableCard } = $props()
  const initial = untrack(() => card)
  const editing = initial !== undefined

  const params = new URLSearchParams(window.location.search)
  const shortcut = /Mac|iPhone|iPad/.test(navigator.platform) ? '⌘' : 'Ctrl'
  let question = $state(initial?.question ?? params.get('question') ?? '')
  let answer = $state(initial?.answer ?? params.get('answer') ?? '')
  let source = $state(
    editing ? (initial.source ?? '') : (params.get('source') ?? ''),
  )
  let tags = $state<string[]>(initial ? [...initial.tags] : [])
  let pendingTag = $state(editing ? '' : (params.get('tag') ?? ''))
  let examples = $state(initial?.examples?.split('\n') ?? [''])
  let keepDetails = $state(true)
  let addedCount = $state(0)
  let side = $state<'front' | 'back'>('front')
  let submitting = $state(false)
  let error = $state<string | null>(null)
  let saved = $state(false)
  let acknowledgementTimer: ReturnType<typeof setTimeout>
  let questionImage = $state<File | null>(null)
  let answerImage = $state<File | null>(null)
  let questionFilename = $state(initial?.question_img ?? null)
  let answerFilename = $state(initial?.img ?? null)
  let questionPreview = $state('')
  let answerPreview = $state('')
  let questionImageInput: ReturnType<typeof ImageInput>
  let answerImageInput: ReturnType<typeof ImageInput>
  let questionField: HTMLTextAreaElement
  let form: HTMLFormElement
  let dragSide = $state<string | null>(null)
  let filledExamples = $derived(
    examples.map((value) => value.trim()).filter(Boolean),
  )

  commitTags()
  onDestroy(() => clearTimeout(acknowledgementTimer))
  onMount(() => on(form, 'keydown', formKeydown))

  async function saveCard(event: SubmitEvent) {
    event.preventDefault()
    if (submitting || !question.trim()) return
    commitTags()
    const request = {
      question,
      answer,
      examples: filledExamples.join('\n') || null,
      source: source || null,
      tags: [...new Set(tags.map((tag) => tag.trim()).filter(Boolean))],
    }
    submitting = true
    error = null
    saved = false
    clearTimeout(acknowledgementTimer)

    try {
      // ponytail: failed card creation can leave unused uploads; add cleanup when media management is needed.
      const question_img_fname = questionImage
        ? await uploadImage(questionImage)
        : questionFilename
      const answer_img_fname = answerImage
        ? await uploadImage(answerImage)
        : answerFilename
      const response = await fetch(
        editing ? `/api/cards/${initial.id}` : '/api/cards',
        {
          method: editing ? 'PUT' : 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            ...request,
            question_img_fname,
            answer_img_fname,
          }),
        },
      )
      if (!response.ok)
        throw new Error(
          `Failed to ${editing ? 'save' : 'create'} card (HTTP ${response.status})`,
        )
      if (editing) {
        questionFilename = question_img_fname
        answerFilename = answer_img_fname
        questionImage = null
        answerImage = null
      } else {
        resetFields(keepDetails)
        addedCount += 1
      }
      saved = true
      acknowledgementTimer = setTimeout(() => (saved = false), 3000)
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Failed to save card'
    } finally {
      submitting = false
    }
    if (saved) {
      await tick()
      questionField.focus()
    }
  }

  async function uploadImage(file: File | null): Promise<string | null> {
    if (!file) return null
    const data = new FormData()
    data.append('image', file)
    const response = await fetch('/api/media', { method: 'POST', body: data })
    if (!response.ok)
      throw new Error(
        `Failed to upload image: ${(await response.text()) || response.status}`,
      )
    const result: { filename: string } = await response.json()
    return result.filename
  }

  function resetFields(keep: boolean) {
    question = ''
    answer = ''
    examples = ['']
    pendingTag = ''
    side = 'front'
    questionImageInput.clear()
    answerImageInput.clear()
    if (!keep) {
      source = ''
      tags = []
    }
    error = null
    saved = false
    clearTimeout(acknowledgementTimer)
  }

  function commitTags(raw = pendingTag) {
    for (const tag of raw
      .split(',')
      .map((value) => value.trim())
      .filter(Boolean)) {
      if (
        !tags.some((existing) => existing.toLowerCase() === tag.toLowerCase())
      )
        tags.push(tag)
    }
    pendingTag = ''
  }

  function splitTags(event: Event & { currentTarget: HTMLInputElement }) {
    const parts = event.currentTarget.value.split(',')
    if (parts.length > 1) {
      const remainder = parts.pop()!
      commitTags(parts.join(','))
      pendingTag = remainder
    }
  }

  function tagKeydown(event: KeyboardEvent) {
    if (event.isComposing) return
    if (event.key === 'Enter' && !event.metaKey && !event.ctrlKey) {
      event.preventDefault()
      commitTags()
    } else if (event.key === 'Backspace' && !pendingTag) {
      tags.pop()
    }
  }

  async function addExample(at = examples.length) {
    examples.splice(at, 0, '')
    await tick()
    form.querySelector<HTMLInputElement>(`#example-${at}`)?.focus()
  }

  function removeExample(index: number) {
    examples.splice(index, 1)
    if (!examples.length) examples = ['']
  }

  function formKeydown(event: KeyboardEvent) {
    if (
      !event.isComposing &&
      (event.metaKey || event.ctrlKey) &&
      event.key === 'Enter'
    ) {
      event.preventDefault()
      if (!submitting) {
        commitTags()
        form.requestSubmit()
      }
    }
  }

  function dragOver(event: DragEvent, target: string) {
    if (!submitting && event.dataTransfer?.types.includes('Files')) {
      event.preventDefault()
      dragSide = target
    }
  }

  function dropImage(event: DragEvent, target: 'front' | 'back') {
    event.preventDefault()
    dragSide = null
    const control = target === 'front' ? questionImageInput : answerImageInput
    control.selectFile(event.dataTransfer?.files[0])
  }
</script>

<div class="page">
  <div class="wrap">
    <header class="header">
      {#if editing}
        <div class="header-left">
          <a class="back-link" href="/list-cards"
            ><ChevronLeft size={16} aria-hidden="true" />Back to cards</a
          >
          <h1>Edit card</h1>
        </div>
      {/if}
      {#if addedCount}<span class="session-pill"
          >{addedCount}
          {addedCount === 1 ? 'card' : 'cards'} added this session</span
        >{/if}
    </header>

    <div class="layout">
      <form
        class="form-col"
        bind:this={form}
        onsubmit={saveCard}
        aria-busy={submitting}
      >
        <fieldset disabled={submitting}>
          <section
            class="panel"
            class:drag-over={dragSide === 'front'}
            aria-label="Question panel"
            ondragover={(event) => dragOver(event, 'front')}
            ondragleave={() => (dragSide = null)}
            ondrop={(event) => dropImage(event, 'front')}
          >
            <div class="panel-head">
              <div>
                <label for="question">Question</label>
                <span class="req" aria-hidden="true">*</span>
              </div>
              <span class="side-badge front">Front</span>
            </div>
            <textarea
              id="question"
              name="question"
              class="field question"
              rows="3"
              required
              bind:this={questionField}
              bind:value={question}
              placeholder="What should you be asked? e.g. “What does ‘idempotent’ mean?”"
              onpaste={(event) => questionImageInput.paste(event)}></textarea>
            <ImageInput
              label="Question image"
              bind:this={questionImageInput}
              bind:file={questionImage}
              bind:filename={questionFilename}
              bind:preview={questionPreview}
              disabled={submitting}
            />
          </section>

          <section
            class="panel"
            class:drag-over={dragSide === 'back'}
            aria-label="Answer panel"
            ondragover={(event) => dragOver(event, 'back')}
            ondragleave={() => (dragSide = null)}
            ondrop={(event) => dropImage(event, 'back')}
          >
            <div class="panel-head">
              <label for="answer">Answer</label><span class="side-badge back"
                >Back</span
              >
            </div>
            <textarea
              id="answer"
              name="answer"
              class="field answer"
              rows="4"
              bind:value={answer}
              placeholder="What you want to recall"
              onpaste={(event) => answerImageInput.paste(event)}></textarea>
            <ImageInput
              label="Answer image"
              bind:this={answerImageInput}
              bind:file={answerImage}
              bind:filename={answerFilename}
              bind:preview={answerPreview}
              disabled={submitting}
            />
            <div class="divider"></div>
            <div class="sub-head">
              <span class="label">Examples</span><span class="hint"
                >Shown under the answer</span
              >
            </div>
            <div class="examples">
              {#each examples as _, index}
                <div class="ex-row">
                  <label class="sr-only" for={`example-${index}`}
                    >Example {index + 1}</label
                  >
                  <span class="ex-n" aria-hidden="true">{index + 1}</span>
                  <input
                    id={`example-${index}`}
                    class="field"
                    bind:value={examples[index]}
                    placeholder="A sentence that uses it"
                    onkeydown={(event) => {
                      if (
                        event.key === 'Enter' &&
                        !event.metaKey &&
                        !event.ctrlKey &&
                        !event.isComposing
                      ) {
                        event.preventDefault()
                        void addExample(index + 1)
                      }
                    }}
                  />
                  <button
                    type="button"
                    class="icon-x"
                    aria-label={`Remove example ${index + 1}`}
                    onclick={() => removeExample(index)}
                    ><X size={14} aria-hidden="true" /></button
                  >
                </div>
              {/each}
            </div>
            <div>
              <button
                type="button"
                class="ghost accent"
                onclick={() => addExample()}
                ><Plus size={18} aria-hidden="true" />Add example</button
              >
            </div>
          </section>

          <section class="panel details" aria-label="Card details">
            <div>
              <label for="source">Source</label>
              <div class="field src">
                <Link size={18} aria-hidden="true" /><input
                  id="source"
                  class="bare"
                  bind:value={source}
                  placeholder="Book, chapter or URL"
                />
              </div>
            </div>
            <div>
              <label for="tags">Tags</label>
              <div class="field tags">
                {#each tags as tag (tag)}<span class="chip"
                    >{tag}<button
                      type="button"
                      aria-label={`Remove tag ${tag}`}
                      onclick={() =>
                        (tags = tags.filter((value) => value !== tag))}
                      ><X size={12} aria-hidden="true" /></button
                    ></span
                  >{/each}
                <input
                  id="tags"
                  class="bare"
                  bind:value={pendingTag}
                  oninput={splitTags}
                  onkeydown={tagKeydown}
                  onblur={() => commitTags()}
                  placeholder={tags.length
                    ? 'Add another…'
                    : 'e.g. vocabulary, chapter-3'}
                />
              </div>
              <span class="hint">Press Enter or comma to add</span>
            </div>
          </section>

          <div class="actions">
            {#if !editing}<label class="keep" for="keep-details"
                ><input
                  id="keep-details"
                  type="checkbox"
                  bind:checked={keepDetails}
                />Keep source &amp; tags for the next card</label
              >{/if}
            <div class="actions-right">
              {#if saved}<span role="status" class="status"
                  ><Check size={18} aria-hidden="true" />{editing
                    ? 'Card saved'
                    : 'Card added'}</span
                >{/if}
              {#if error}<p role="alert" class="alert">{error}</p>{/if}
              {#if !editing}<button
                  type="button"
                  class="ghost clear"
                  onclick={() => resetFields(false)}>Clear</button
                >{/if}
              <button
                type="submit"
                class="primary"
                disabled={submitting || !question.trim()}
                >{editing
                  ? submitting
                    ? 'Saving…'
                    : 'Save changes'
                  : submitting
                    ? 'Creating…'
                    : 'Create card'}<span class="kbd" aria-hidden="true"
                  >{shortcut} ↵</span
                ></button
              >
            </div>
          </div>
        </fieldset>
      </form>

      <aside class="preview-col" aria-label="Card preview">
        <div class="preview-head">
          <span class="eyebrow">Preview</span>
          <div class="seg" role="group" aria-label="Card side">
            <button
              type="button"
              aria-pressed={side === 'front'}
              onclick={() => (side = 'front')}>Front</button
            ><button
              type="button"
              aria-pressed={side === 'back'}
              onclick={() => (side = 'back')}>Back</button
            >
          </div>
        </div>
        <button
          type="button"
          class="card"
          aria-label="Flip preview card"
          onclick={() => (side = side === 'front' ? 'back' : 'front')}
        >
          <span
            class="card-kicker"
            class:front={side === 'front'}
            class:back={side === 'back'}
            >{side === 'front' ? 'Question' : 'Answer'}</span
          >
          {#if side === 'front'}
            {#if questionPreview}<img
                class="card-img"
                src={questionPreview}
                alt=""
              />{/if}
            <span class="card-q" class:card-empty={!question.trim()}
              >{question.trim()
                ? question
                : 'Your question will appear here'}</span
            >
          {:else}
            {#if answerPreview}<img
                class="card-img"
                src={answerPreview}
                alt=""
              />{/if}
            <span class="card-a" class:card-empty={!answer.trim()}
              >{answer.trim() ? answer : 'The answer goes here'}</span
            >
            {#if filledExamples.length}<span class="card-ex"
                >{#each filledExamples as example}<span>{example}</span
                  >{/each}</span
              >{/if}
          {/if}
          <span class="card-foot"
            >{#each tags as tag}<span class="card-tag">#{tag}</span
              >{/each}{#if source.trim()}<span class="card-src">{source}</span
              >{/if}</span
          >
        </button>
        <span class="flip-hint">Click the card to flip it</span>
      </aside>
    </div>
  </div>
</div>

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
    gap: 28px;
  }
  .header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
  }
  .header-left {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .back-link {
    font-size: 14px;
    font-weight: 500;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--color-accent);
  }
  .back-link:hover {
    color: var(--color-accent-hover);
  }
  h1 {
    font-size: 32px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .session-pill {
    font-size: 14px;
    color: var(--color-subtle);
    background: var(--color-surface);
    border: 1px solid var(--color-line);
    border-radius: 999px;
    padding: 8px 14px;
  }
  .layout {
    display: flex;
    flex-wrap: wrap;
    gap: 28px;
    align-items: flex-start;
  }
  .form-col {
    flex: 999 1 560px;
    min-width: 0;
  }
  fieldset {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .preview-col {
    flex: 1 1 340px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .panel {
    background: var(--color-surface);
    border: 1px solid var(--color-line);
    border-radius: 16px;
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .panel.drag-over {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px rgb(53 84 209 / 0.18);
  }
  .panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  label,
  .label {
    font-size: 15px;
    font-weight: 600;
  }
  .req {
    color: var(--color-danger);
  }
  .side-badge {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    padding: 4px 10px;
    border-radius: 999px;
  }
  .front {
    color: var(--color-accent);
  }
  .back {
    color: var(--color-back);
  }
  .side-badge.front {
    background: rgb(53 84 209 / 0.1);
  }
  .side-badge.back {
    background: var(--color-back-soft);
  }
  .field {
    border: 1px solid var(--color-field);
    border-radius: 10px;
    background: var(--color-surface);
    transition:
      border-color 0.15s,
      box-shadow 0.15s;
  }
  .field:hover {
    border-color: #a9b0bc;
  }
  .field:focus,
  .field:focus-within {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px rgb(53 84 209 / 0.18);
  }
  .field::placeholder,
  .bare::placeholder {
    color: var(--color-placeholder);
    opacity: 1;
  }
  .bare {
    border: 0;
    background: transparent;
    outline: none;
  }
  textarea.field {
    width: 100%;
    padding: 12px 14px;
    resize: vertical;
    line-height: 1.5;
  }
  .question {
    font-size: 17px;
    min-height: 92px;
  }
  textarea.answer {
    font-size: 16px;
    line-height: 1.55;
    min-height: 112px;
  }
  button {
    cursor: pointer;
  }
  button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 3px;
  }
  button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .ghost {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-height: 44px;
    padding: 0 14px;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-ink-2);
  }
  .ghost:hover {
    background: var(--color-chip);
  }
  .ghost.accent {
    color: var(--color-accent);
    padding-left: 10px;
  }
  .divider {
    height: 1px;
    background: var(--color-divider);
    margin: 6px 0;
  }
  .sub-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
  }
  .hint {
    font-size: 13px;
    color: var(--color-hint);
  }
  .examples {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-bottom: 8px;
  }
  .ex-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .ex-n {
    width: 24px;
    flex: none;
    text-align: right;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-placeholder);
  }
  .ex-row input {
    flex: 1;
    min-width: 0;
    height: 44px;
    padding: 0 12px;
    font-size: 15px;
  }
  .icon-x {
    width: 44px;
    height: 44px;
    flex: none;
    border-radius: 10px;
    color: var(--color-subtle);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .icon-x:hover,
  .chip button:hover {
    background: var(--color-line);
  }
  .details {
    flex-direction: row;
    flex-wrap: wrap;
    gap: 18px;
  }
  .details > div {
    flex: 1 1 240px;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .src {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 48px;
    padding: 0 12px;
    color: var(--color-hint);
  }
  .src input {
    flex: 1;
    min-width: 0;
    height: 100%;
    font-size: 15px;
    color: var(--color-ink);
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    min-height: 48px;
    padding: 6px 10px;
  }
  .tags input {
    flex: 1;
    min-width: 120px;
    width: 0;
    height: 34px;
    font-size: 15px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    min-height: 32px;
    max-width: 100%;
    overflow-wrap: anywhere;
    padding: 0 2px 0 12px;
    background: var(--color-chip);
    border-radius: 999px;
    font-size: 14px;
    font-weight: 500;
    color: #2b303a;
  }
  .chip button {
    width: 28px;
    height: 28px;
    flex: none;
    border-radius: 999px;
    color: var(--color-subtle);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 4px 2px;
  }
  .keep {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    min-height: 44px;
    font-size: 14px;
    font-weight: 400;
    color: var(--color-ink-2);
    cursor: pointer;
  }
  .keep input {
    width: 18px;
    height: 18px;
    flex: none;
    accent-color: var(--color-accent);
  }
  .actions-right {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-success);
  }
  .alert {
    font-size: 14px;
    color: var(--color-danger);
    overflow-wrap: anywhere;
  }
  .ghost.clear {
    min-height: 48px;
    padding: 0 18px;
    font-size: 15px;
  }
  .primary {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    min-height: 48px;
    padding: 0 20px;
    border-radius: 10px;
    background: var(--color-accent);
    color: white;
    font-size: 15px;
    font-weight: 600;
  }
  .primary:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .kbd {
    font-size: 12px;
    font-weight: 600;
    background: rgb(255 255 255 / 0.2);
    border-radius: 6px;
    padding: 3px 6px;
  }
  .preview-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .eyebrow {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-subtle);
  }
  .seg {
    display: flex;
    padding: 3px;
    background: #e4e6ea;
    border-radius: 10px;
  }
  .seg button {
    min-height: 44px;
    padding: 0 14px;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-subtle);
  }
  .seg button[aria-pressed='true'] {
    background: var(--color-surface);
    color: var(--color-ink);
    box-shadow: 0 1px 2px rgb(21 23 28 / 0.12);
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 16px;
    width: 100%;
    min-height: 360px;
    padding: 28px;
    text-align: left;
    border: 1px solid var(--color-line);
    border-radius: 20px;
    background: var(--color-surface);
    box-shadow: 0 6px 20px rgb(21 23 28 / 0.07);
    transition:
      transform 0.2s,
      box-shadow 0.2s;
    overflow-wrap: anywhere;
  }
  .card:hover {
    transform: translateY(-2px);
    box-shadow: 0 14px 34px rgb(21 23 28 / 0.12);
  }
  .card-kicker {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .card-img {
    width: 100%;
    max-height: 180px;
    object-fit: contain;
    border-radius: 10px;
    background: var(--color-page);
  }
  .card-q {
    font-family: 'Newsreader', Georgia, serif;
    font-size: 26px;
    line-height: 1.3;
    white-space: pre-wrap;
  }
  .card-a {
    font-family: 'Newsreader', Georgia, serif;
    font-size: 21px;
    line-height: 1.45;
    white-space: pre-wrap;
  }
  .card-empty {
    color: var(--color-placeholder);
  }
  .card-ex {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 4px;
  }
  .card-ex span {
    font-family: 'Newsreader', Georgia, serif;
    font-style: italic;
    font-size: 17px;
    line-height: 1.45;
    color: var(--color-ink-2);
    padding-left: 14px;
    border-left: 2px solid var(--color-divider);
    white-space: pre-wrap;
  }
  .card-foot {
    margin-top: auto;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    width: 100%;
  }
  .card-tag {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-subtle);
    background: var(--color-page);
    border-radius: 999px;
    padding: 4px 10px;
  }
  .card-src {
    font-size: 13px;
    color: var(--color-hint);
    margin-left: auto;
  }
  .flip-hint {
    font-size: 13px;
    color: var(--color-hint);
    text-align: center;
  }
</style>
