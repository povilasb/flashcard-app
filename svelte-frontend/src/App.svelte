<script lang="ts">
  import ListCards from '$lib/components/ListCards.svelte'
  import ReviewCards from '$lib/components/ReviewCards.svelte'
  import CardEditor from '$lib/components/CardEditor.svelte'
  import EditCard from '$lib/components/EditCard.svelte'
  import LanguageOverview from '$lib/components/languages/Overview.svelte'
  import Vocabulary from '$lib/components/languages/Vocabulary.svelte'
  import GenerateSentence from '$lib/components/languages/GenerateSentence.svelte'
  import WriteStory from '$lib/components/languages/WriteStory.svelte'

  const path = window.location.pathname
  const isListPage = path === '/' || path === '/list-cards'
  const reviewTag = path.match(/^\/review-cards\/(.+?)\/?$/)?.[1]
  const viewId = path.match(/^\/cards\/(\d+)\/?$/)?.[1]
  const isReviewPage = path === '/review-cards' || !!reviewTag
  const isAddPage = path === '/add-card'
  const editId = path.match(/^\/cards\/edit\/(\d+)\/?$/)?.[1]
  const isEditorPage = isAddPage || !!editId
  const isLanguagePage =
    path === '/learn-languages' || path.startsWith('/learn-languages/')
  const isDesignedPage = isEditorPage || isListPage || isLanguagePage
</script>

<div
  class="flex min-h-screen"
  class:flex-wrap={isDesignedPage}
  class:max-sm:content-start={isDesignedPage}
>
  <nav
    class="w-64 shrink-0 border-r border-gray-200 bg-gray-100 p-4"
    class:max-sm:w-full={isDesignedPage}
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
      <a
        href="/learn-languages"
        aria-current={isLanguagePage ? 'page' : undefined}
        class="block rounded px-4 py-2 text-gray-700 hover:bg-gray-200 aria-[current=page]:bg-gray-200"
        >Learn languages</a
      >
    </div>
  </nav>

  <main
    class="min-w-0 flex-1"
    class:p-4={!isDesignedPage}
    class:max-sm:basis-full={isDesignedPage}
  >
    {#if isAddPage}
      <CardEditor />
    {:else if editId}
      <EditCard cardId={editId} />
    {:else if isReviewPage}
      <ReviewCards
        tag={reviewTag ? decodeURIComponent(reviewTag) : undefined}
      />
    {:else if viewId}
      <ReviewCards cardId={viewId} />
    {:else if isListPage}
      <ListCards />
    {:else if isLanguagePage}
      <div class="language-page min-h-full bg-page p-6 text-ink sm:p-10">
        <div class="mx-auto max-w-[1160px]">
          {#if path !== '/learn-languages'}<a
              class="mb-5 inline-block text-accent hover:underline"
              href="/learn-languages">Back to languages</a
            >{/if}
          {#if path === '/learn-languages'}<h1 class="mb-5 text-3xl font-bold">
              Learn languages
            </h1>
            <LanguageOverview />
          {:else if path === '/learn-languages/vocabulary'}<Vocabulary />
          {:else if path === '/learn-languages/generate-sentence'}<GenerateSentence
            />
          {:else if path === '/learn-languages/write-story'}<WriteStory />
          {:else}<h1>Page not found.</h1>{/if}
        </div>
      </div>
    {:else}
      <h1>Page not found.</h1>
    {/if}
  </main>
</div>

<style>
  .language-page {
    font-family: 'Figtree', system-ui, sans-serif;
    :global(button:focus-visible),
    :global(a:focus-visible),
    :global(input:focus-visible) {
      outline: 2px solid var(--color-accent);
      outline-offset: 2px;
    }
  }
</style>
