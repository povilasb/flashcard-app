<script lang="ts">
  import { onMount } from 'svelte'
  import * as Table from '$lib/components/ui/table/index.js'

  type Card = {
    id: number
    question: string
    answer: string
    tags: string[]
    last_reviewed: string
  }

  let cards = $state<Card[]>([])
  let loading = $state(true)
  let error = $state<string | null>(null)

  onMount(() => {
    void loadCards()
  })

  async function loadCards() {
    try {
      const response = await fetch('/api/cards')
      if (!response.ok) {
        throw new Error(`Failed to load cards (HTTP ${response.status})`)
      }
      cards = await response.json()
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Failed to load cards'
    } finally {
      loading = false
    }
  }

  function formatReviewedAt(value: string) {
    const date = new Date(value)
    return Number.isNaN(date.getTime())
      ? 'Unknown'
      : date.toISOString().slice(0, 16).replace('T', ' ')
  }
</script>

<section
  class="container mx-auto p-4"
  aria-label="Flashcards"
  aria-busy={loading}
>
  {#if loading}
    <p role="status">Loading cards...</p>
  {:else if error}
    <p role="alert" class="text-red-600">{error}</p>
  {:else}
    <h1 class="mb-4 text-2xl font-bold">Total: {cards.length}</h1>
    <Table.Root class="min-w-full border border-gray-300 bg-white">
      <Table.Header>
        <Table.Row class="bg-gray-100 hover:bg-gray-100">
          <Table.Head scope="col" class="border px-4 py-2">Question</Table.Head>
          <Table.Head scope="col" class="border px-4 py-2">Answer</Table.Head>
          <Table.Head scope="col" class="border px-4 py-2">Tags</Table.Head>
          <Table.Head scope="col" class="border px-4 py-2"
            >Last Reviewed</Table.Head
          >
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#each cards as card (card.id)}
          <Table.Row class="hover:bg-gray-50">
            <Table.Cell class="border px-4 py-2 whitespace-normal"
              ><a
                href={`/cards/edit/${card.id}`}
                class="block text-blue-700 hover:underline focus-visible:underline"
                >{card.question || 'Untitled card'}</a
              ></Table.Cell
            >
            <Table.Cell class="border px-4 py-2 whitespace-normal"
              >{card.answer}</Table.Cell
            >
            <Table.Cell class="border px-4 py-2 whitespace-normal"
              >{card.tags.join(', ')}</Table.Cell
            >
            <Table.Cell class="border px-4 py-2">
              <time datetime={card.last_reviewed}
                >{formatReviewedAt(card.last_reviewed)}</time
              >
            </Table.Cell>
          </Table.Row>
        {:else}
          <Table.Row>
            <Table.Cell colspan={4} class="py-8 text-center"
              >No cards yet.</Table.Cell
            >
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  {/if}
</section>
