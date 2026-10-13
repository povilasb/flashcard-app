<script lang="ts">
  import DOMPurify from 'dompurify'

  let { text }: { text: string } = $props()
  let rendered = $derived(render(text))

  async function render(text: string) {
    const response = await fetch('/api/rich-text/render', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text }),
    })
    if (!response.ok) throw new Error('Failed to render text')
    const result: { html: string } = await response.json()
    return DOMPurify.sanitize(result.html, {
      USE_PROFILES: { html: true, mathMl: true },
    })
  }
</script>

{#await rendered}
  <div class="whitespace-pre-wrap">{text}</div>
{:then html}
  <div class="rich-text">{@html html}</div>
{:catch}
  <div class="whitespace-pre-wrap">{text}</div>
  <p role="alert" class="text-sm text-red-600">Unable to format text.</p>
{/await}

<style>
  .rich-text {
    :global(ol) {
      list-style: decimal inside;
    }
    :global(ul) {
      list-style: disc inside;
    }
    :global(pre) {
      overflow-x: auto;
    }
  }
</style>
