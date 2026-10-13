<script lang="ts">
  import ListCards from '$lib/components/ListCards.svelte'
  import ReviewCards from '$lib/components/ReviewCards.svelte'
  import AddCard from '$lib/components/AddCard.svelte'

  const path = window.location.pathname
  const isListPage = path === '/' || path === '/list-cards'
  const isReviewPage = path === '/review-cards'
  const isAddPage = path === '/add-card'
</script>

<div class="flex min-h-screen">
  <nav
    class="w-64 shrink-0 border-r border-gray-200 bg-gray-100 p-4"
    aria-label="Main navigation"
  >
    <div class="space-y-2">
      <a
        href="/add-card"
        aria-current={isAddPage ? 'page' : undefined}
        class="block rounded px-4 py-2 text-gray-700 hover:bg-gray-200 aria-[current=page]:bg-gray-200"
        >Add new card</a
      >
      <a
        href="/review-cards"
        aria-current={isReviewPage ? 'page' : undefined}
        class="block rounded px-4 py-2 text-gray-700 hover:bg-gray-200 aria-[current=page]:bg-gray-200"
        >Review cards</a
      >
      <a
        href="/list-cards"
        aria-current={isListPage ? 'page' : undefined}
        class="block rounded px-4 py-2 text-gray-700 hover:bg-gray-200 aria-[current=page]:bg-gray-200"
        >List cards</a
      >
    </div>
  </nav>

  <main class="min-w-0 flex-1 p-4">
    {#if isAddPage}
      <AddCard />
    {:else if isReviewPage}
      <ReviewCards />
    {:else if isListPage}
      <ListCards />
    {:else}
      <h1>Page not found.</h1>
    {/if}
  </main>
</div>
