<script lang="ts">
  import { Layers, Moon, Plus, Sun } from 'lucide-svelte'
  import ListCards from '$lib/components/ListCards.svelte'
  import ReviewCards from '$lib/components/ReviewCards.svelte'
  import CardEditor from '$lib/components/CardEditor.svelte'
  import EditCard from '$lib/components/EditCard.svelte'
  import LanguageOverview from '$lib/components/languages/Overview.svelte'
  import Vocabulary from '$lib/components/languages/Vocabulary.svelte'
  import GenerateSentence from '$lib/components/languages/GenerateSentence.svelte'
  import WriteStory from '$lib/components/languages/WriteStory.svelte'

  let initialDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  try {
    const saved = localStorage.getItem('theme')
    if (saved === 'dark' || saved === 'light') initialDark = saved === 'dark'
  } catch {
    // Storage may be unavailable; the toggle still works for this page.
  }
  document.documentElement.dataset.theme = initialDark ? 'dark' : 'light'
  let dark = $state(initialDark)

  function toggleTheme() {
    dark = !dark
    document.documentElement.dataset.theme = dark ? 'dark' : 'light'

    try {
      localStorage.setItem('theme', dark ? 'dark' : 'light')
    } catch {
      // Storage may be unavailable; the toggle still works for this page.
    }
  }

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

<div class="flex min-h-screen max-[760px]:flex-col">
  <aside
    class="w-64 shrink-0 border-r border-line bg-chip p-4 max-[760px]:grid max-[760px]:w-full max-[760px]:grid-cols-[minmax(0,1fr)_44px_44px] max-[760px]:items-center max-[760px]:gap-x-3 max-[760px]:gap-y-2 max-[760px]:border-r-0 max-[760px]:border-b max-[760px]:bg-surface max-[760px]:px-3 max-[760px]:py-2.5 max-[760px]:font-['Figtree',system-ui,sans-serif] [&_a:focus-visible]:outline-2 [&_a:focus-visible]:outline-offset-2 [&_a:focus-visible]:outline-accent"
  >
    <a
      class="hidden min-h-11 items-center gap-2.5 rounded-[10px] text-base font-bold text-ink max-[760px]:flex"
      href="/"
      aria-label="Flashcards home"
    >
      <span
        class="flex size-8 items-center justify-center rounded-[9px] bg-accent text-on-accent"
        ><Layers size={18} aria-hidden="true" /></span
      >
      Flashcards
    </a>
    <button
      class="mb-4 inline-flex size-11 cursor-pointer items-center justify-center rounded text-ink-2 hover:bg-line focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent max-[760px]:col-start-3 max-[760px]:row-start-1 max-[760px]:m-0 max-[760px]:rounded-[10px]"
      aria-label={dark ? 'Switch to light theme' : 'Switch to dark theme'}
      title={dark ? 'Switch to light theme' : 'Switch to dark theme'}
      onclick={toggleTheme}
    >
      {#if dark}<Sun size={20} aria-hidden="true" />{:else}<Moon
          size={20}
          aria-hidden="true"
        />{/if}
    </button>
    <a
      href="/add-card"
      aria-label="Add new card"
      aria-current={isEditorPage ? 'page' : undefined}
      class="mb-2 block rounded px-4 py-2 text-ink-2 hover:bg-line aria-[current=page]:bg-line max-[760px]:col-start-2 max-[760px]:row-start-1 max-[760px]:m-0 max-[760px]:flex max-[760px]:h-11 max-[760px]:items-center max-[760px]:justify-center max-[760px]:rounded-[10px] max-[760px]:bg-accent max-[760px]:p-0 max-[760px]:text-on-accent max-[760px]:hover:bg-accent-hover max-[760px]:aria-[current=page]:bg-accent max-[760px]:aria-[current=page]:outline-3 max-[760px]:aria-[current=page]:outline-accent-soft"
    >
      <span class="hidden max-[760px]:inline-flex"
        ><Plus size={20} aria-hidden="true" /></span
      >
      <span class="max-[760px]:hidden">Add new card</span>
    </a>
    <nav
      class="space-y-2 max-[760px]:col-span-full max-[760px]:flex max-[760px]:min-w-0 max-[760px]:gap-1 max-[760px]:space-y-0 max-[760px]:overflow-x-auto max-[760px]:p-1 max-[760px]:[&_a]:flex max-[760px]:[&_a]:min-h-11 max-[760px]:[&_a]:flex-[1_0_auto] max-[760px]:[&_a]:items-center max-[760px]:[&_a]:justify-center max-[760px]:[&_a]:rounded-[10px] max-[760px]:[&_a]:px-3 max-[760px]:[&_a]:py-0 max-[760px]:[&_a]:text-sm max-[760px]:[&_a]:font-medium max-[760px]:[&_a[aria-current=page]]:bg-accent-soft max-[760px]:[&_a[aria-current=page]]:font-semibold max-[760px]:[&_a[aria-current=page]]:text-accent"
      aria-label="Main navigation"
    >
      <a
        href="/review-cards"
        aria-current={isReviewPage || viewId ? 'page' : undefined}
        class="block rounded px-4 py-2 text-ink-2 hover:bg-line aria-[current=page]:bg-line"
        ><span class="max-[760px]:hidden">Review cards</span><span
          class="hidden max-[760px]:inline-flex">Review</span
        ></a
      >
      <a
        href="/list-cards"
        aria-current={isListPage ? 'page' : undefined}
        class="block rounded px-4 py-2 text-ink-2 hover:bg-line aria-[current=page]:bg-line"
        ><span class="max-[760px]:hidden">List cards</span><span
          class="hidden max-[760px]:inline-flex">Cards</span
        ></a
      >
      <a
        href="/learn-languages"
        aria-current={isLanguagePage ? 'page' : undefined}
        class="block rounded px-4 py-2 text-ink-2 hover:bg-line aria-[current=page]:bg-line"
        >Learn languages</a
      >
    </nav>
  </aside>

  <main
    class={[
      'min-w-0 flex-1',
      !isDesignedPage &&
        'p-4 max-[760px]:px-4 max-[760px]:pt-6 max-[760px]:pb-12',
    ]}
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
