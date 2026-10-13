<script lang="ts">
  import { onMount } from 'svelte'
  import CardEditor, { type EditableCard } from './CardEditor.svelte'

  let { cardId }: { cardId: string } = $props()
  let card = $state<EditableCard | null>(null)
  let error = $state<string | null>(null)

  onMount(() => {
    void loadCard()
  })

  async function loadCard() {
    error = null
    try {
      const response = await fetch(`/api/cards/${cardId}`)
      if (!response.ok)
        throw new Error(
          response.status === 404
            ? 'Card not found.'
            : `Failed to load card (HTTP ${response.status})`,
        )
      card = await response.json()
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Failed to load card'
    }
  }
</script>

{#if card}
  <CardEditor {card} />
{:else if error}
  <div class="p-6">
    <p role="alert">{error}</p>
    <button class="mt-4 underline" onclick={loadCard}>Retry</button><a
      class="ml-4 underline"
      href="/list-cards">Back to cards</a
    >
  </div>
{:else}
  <p role="status" class="p-6">Loading card...</p>
{/if}
