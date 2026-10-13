import { expect, test, type Page } from '@playwright/test'

const vocabulary = [
  { word: 'café', translation: '', created_at: '2026-10-13T09:00:00Z' },
]
const generated = {
  sentence: { text: '¿Café & pan?', new_word: 'pan', translation: 'bread' },
  language: 'spanish',
}

test('overview links and vocabulary CRUD preserve drafts on failure', async ({
  page,
}) => {
  let words = structuredClone(vocabulary)
  let failSave = true
  await page.route('**/api/languages/vocabulary*', async (route) => {
    const req = route.request()
    const data =
      req.method() === 'POST' || req.method() === 'PUT'
        ? req.postDataJSON()
        : null
    if (req.method() === 'GET') return route.fulfill({ json: words })
    if (req.method() === 'POST') {
      if (!words.some((item) => item.word === data.word))
        words.push({ ...data, created_at: vocabulary[0].created_at })
      return route.fulfill({ status: 201 })
    }
    if (req.method() === 'PUT') {
      if (failSave)
        return route.fulfill({
          status: 500,
          json: { error: { message: 'Save failed' } },
        })
      words = words.map((item) =>
        item.word === data.word
          ? { ...item, translation: data.translation }
          : item,
      )
      return route.fulfill({ status: 204 })
    }
    words = words.filter(
      (item) => item.word !== new URL(req.url()).searchParams.get('word'),
    )
    return route.fulfill({ status: 204 })
  })
  await page.goto('/learn-languages')

  await page.getByRole('link', { name: 'Vocabulary', exact: true }).click()

  await expect(page.getByText('Total: 1', { exact: true })).toBeVisible()

  await page.getByLabel('Translation for café', { exact: true }).fill('coffee')
  await page.getByLabel('Translation for café', { exact: true }).press('Tab')

  await expect(page.getByRole('alert')).toContainText('Save failed')
  await expect(
    page.getByLabel('Translation for café', { exact: true }),
  ).toHaveValue('coffee')

  failSave = false
  await page.getByRole('button', { name: 'Retry save' }).click()
  await page.getByLabel('Word', { exact: true }).fill('por favor')
  await page.getByLabel('Translation', { exact: true }).fill('please')
  await page.getByRole('button', { name: 'Add word', exact: true }).click()

  await expect(page.getByText('Total: 2', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Word', { exact: true })).toHaveValue('')
  await expect(
    page.getByLabel('Translation for café', { exact: true }),
  ).toHaveValue('coffee')

  page.once('dialog', (dialog) => dialog.dismiss())
  await page
    .getByRole('button', { name: 'Delete word: café', exact: true })
    .click()

  await expect(page.getByText('Total: 2', { exact: true })).toBeVisible()

  page.once('dialog', (dialog) => dialog.accept())
  await page
    .getByRole('button', { name: 'Delete word: café', exact: true })
    .click()

  await expect(page.getByText('Total: 1', { exact: true })).toBeVisible()
})

test('import is guarded while pending and refreshes vocabulary', async ({
  page,
}) => {
  let finish!: () => void
  const pending = new Promise<void>((resolve) => (finish = resolve))
  let imports = 0
  await page.route('**/api/languages/vocabulary', (route) =>
    route.fulfill({ json: imports ? vocabulary : [] }),
  )
  await page.route(
    '**/api/languages/vocabulary/from-flashcards',
    async (route) => {
      imports++
      await pending
      await route.fulfill({ status: 204 })
    },
  )
  await page.goto('/learn-languages/vocabulary')

  await page.getByRole('button', { name: 'Add from flashcards' }).click()

  await expect(page.getByRole('button', { name: 'Working…' })).toBeDisabled()

  finish()

  await expect(page.getByText('Total: 1', { exact: true })).toBeVisible()
  expect(imports).toBe(1)
})

test('sentence generation is explicit and hands encoded prefills to the existing editor', async ({
  page,
}) => {
  let calls = 0
  let fail = false
  await page.route('**/api/languages/sentence', (route) => {
    calls++
    return route.fulfill(
      fail
        ? { status: 502, json: { error: { message: 'Provider failed' } } }
        : { json: generated },
    )
  })
  await page.goto('/learn-languages/generate-sentence')

  expect(calls).toBe(0)
  await expect(
    page.getByRole('link', { name: 'Create flashcard' }),
  ).toHaveCount(0)

  await page
    .getByRole('button', { name: 'Generate Sentence', exact: true })
    .click()

  await expect(
    page.getByRole('cell', { name: generated.sentence.text, exact: true }),
  ).toBeVisible()

  fail = true
  await page
    .getByRole('button', { name: 'Generate Sentence', exact: true })
    .click()

  await expect(page.getByRole('alert')).toContainText('Provider failed')
  await expect(
    page.getByRole('cell', { name: generated.sentence.text, exact: true }),
  ).toBeVisible()

  await page.getByRole('link', { name: 'Create flashcard' }).click()

  await expect(page.getByLabel('Answer', { exact: true })).toHaveValue(
    '¿Café & pan?',
  )
  await expect(page.getByLabel('Question', { exact: true })).toHaveValue('')
  await expect(page.getByLabel('Source', { exact: true })).toHaveValue(
    'learning-languages app',
  )
  await expect(
    page.getByRole('button', { name: 'Remove tag spanish', exact: true }),
  ).toBeVisible()
})

test('story hover falls back for empty translations and ignores late responses', async ({
  page,
}) => {
  let storyCalls = 0
  let finish!: () => void
  const delayed = new Promise<void>((resolve) => (finish = resolve))
  await page.route('**/api/languages/story', (route) => {
    storyCalls++
    return route.fulfill({ json: { story: 'Hola café.', language: 'spanish' } })
  })
  await page.route('**/api/languages/translation?*', async (route) => {
    if (new URL(route.request().url()).searchParams.get('word') === 'hola') {
      await delayed
      return route.fulfill({ json: { translation: 'hello (late)' } })
    }
    return route.fulfill({ json: { translation: '' } })
  })
  await page.route('**/api/languages/translate', (route) =>
    route.fulfill({ json: { translation: 'coffee' } }),
  )
  await page.goto('/learn-languages/write-story')

  await page
    .getByRole('button', { name: 'Translate Hola', exact: true })
    .hover()
  await page
    .getByRole('button', { name: 'Translate café.', exact: true })
    .hover()

  await expect(page.getByRole('status')).toHaveText('coffee')

  finish()
  await page.getByRole('heading').hover()

  await expect(page.getByRole('status')).toHaveCount(0)
  expect(storyCalls).toBe(1)
})

async function selectStory(page: Page) {
  await page.getByLabel('Story text').evaluate((element) => {
    const range = document.createRange()
    range.selectNodeContents(element)
    const selection = window.getSelection()!
    selection.removeAllRanges()
    selection.addRange(range)
    element.dispatchEvent(new PointerEvent('pointerup', { bubbles: true }))
  })
}

test('selection clears safely while pending and creates an English-question card', async ({
  page,
}) => {
  let finish!: () => void
  const delayed = new Promise<void>((resolve) => (finish = resolve))
  let requests = 0
  await page.route('**/api/languages/story', (route) =>
    route.fulfill({ json: { story: '¿Café & pan?', language: 'spanish' } }),
  )
  await page.route('**/api/languages/translate', async (route) => {
    if (++requests === 1) await delayed
    await route.fulfill({ json: { translation: 'Coffee & bread?' } })
  })
  await page.goto('/learn-languages/write-story')
  await expect(page.getByLabel('Story text')).toHaveText('¿Café & pan?')

  await selectStory(page)

  await expect(page.getByText('Translating selection…')).toBeVisible()

  await page.getByRole('button', { name: 'Clear selection' }).click()
  finish()

  await expect(page.getByRole('region', { name: 'Selected text' })).toHaveCount(
    0,
  )

  await selectStory(page)
  await page.getByRole('link', { name: 'Create flashcard' }).click()

  await expect(page.getByLabel('Question', { exact: true })).toHaveValue(
    'Coffee & bread?',
  )
  await expect(page.getByLabel('Answer', { exact: true })).toHaveValue(
    '¿Café & pan?',
  )
})

test('story failures are visible and retry generates once more', async ({
  page,
}) => {
  let fail = true
  await page.route('**/api/languages/story', (route) =>
    route.fulfill(
      fail
        ? { status: 504, json: { error: { message: 'Timed out' } } }
        : { json: { story: 'Hola.', language: 'spanish' } },
    ),
  )
  await page.goto('/learn-languages/write-story')

  await expect(page.getByRole('alert')).toContainText('Timed out')

  fail = false
  await page.getByRole('button', { name: 'Retry story' }).click()

  await expect(page.getByLabel('Story text')).toHaveText('Hola.')
})

test('sentence submission is disabled until its request finishes', async ({
  page,
}) => {
  let finish!: () => void
  const pending = new Promise<void>((resolve) => (finish = resolve))
  let calls = 0
  await page.route('**/api/languages/sentence', async (route) => {
    calls++
    await pending
    await route.fulfill({ json: generated })
  })
  await page.goto('/learn-languages/generate-sentence')

  await page
    .getByRole('button', { name: 'Generate Sentence', exact: true })
    .click()

  await expect(
    page.getByRole('button', { name: 'Generate Sentence', exact: true }),
  ).toBeDisabled()
  await expect(page.getByRole('status')).toHaveText('Generating sentence…')

  finish()

  await expect(
    page.getByRole('link', { name: 'Create flashcard' }),
  ).toBeVisible()
  expect(calls).toBe(1)
})

test('story supports keyboard lookup and visible selection-translation errors on narrow screens', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.route('**/api/languages/story', (route) =>
    route.fulfill({ json: { story: 'Café.', language: 'spanish' } }),
  )
  await page.route('**/api/languages/translation?*', (route) =>
    route.fulfill({ json: { translation: 'coffee' } }),
  )
  await page.route('**/api/languages/translate', (route) =>
    route.fulfill({
      status: 502,
      json: { error: { message: 'Translation failed' } },
    }),
  )
  await page.goto('/learn-languages/write-story')

  await page
    .getByRole('button', { name: 'Translate Café.', exact: true })
    .focus()

  await expect(page.getByRole('status')).toHaveText('coffee')
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth > innerWidth,
    ),
  ).toBe(false)

  await selectStory(page)

  await expect(page.getByRole('alert')).toContainText('Translation failed')
  await expect(
    page.getByRole('link', { name: 'Create flashcard' }),
  ).toHaveCount(0)
})
