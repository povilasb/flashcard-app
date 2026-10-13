<script lang="ts">
  import { request, flashcardLink } from './api'
  type Result = {
    sentence: { text: string; new_word: string; translation: string }
    language: string
  }
  let result = $state<Result | null>(null)
  let pending = $state(false)
  let error = $state('')
  async function generate() {
    if (pending) return
    pending = true
    error = ''
    try {
      result = await request<Result>('sentence', undefined, 'POST')
    } catch (cause) {
      error = String(cause)
    } finally {
      pending = false
    }
  }
</script>

<section aria-busy={pending}>
  <h1 class="mb-5 text-3xl font-bold">Learn new words</h1>
  <button
    class="rounded bg-accent px-4 py-2 text-on-accent disabled:opacity-50"
    disabled={pending}
    onclick={generate}>Generate Sentence</button
  >
  {#if pending}<p role="status" class="py-3">Generating sentence…</p>{/if}
  {#if error}<p role="alert" class="py-3">{error}</p>{/if}
  <div class="my-5 overflow-x-auto rounded-xl border bg-surface">
    <table class="w-full text-left">
      <thead class="bg-table-head"
        ><tr
          ><th scope="col" class="p-3">New Sentence</th><th
            scope="col"
            class="p-3">New Word</th
          ><th scope="col" class="p-3">Translation</th></tr
        ></thead
      ><tbody
        ><tr
          ><td class="p-3">{result?.sentence.text ?? ''}</td><td class="p-3"
            >{result?.sentence.new_word ?? ''}</td
          ><td class="p-3">{result?.sentence.translation ?? ''}</td></tr
        ></tbody
      >
    </table>
  </div>
  {#if result && !pending}<a
      class="inline-block rounded bg-accent px-4 py-2 text-on-accent"
      href={flashcardLink(result.sentence.text, result.language)}
      >Create flashcard</a
    >{/if}
</section>
