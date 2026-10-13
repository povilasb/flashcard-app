import { expect, test } from '@playwright/test'

test('creates a card with filenames and normalized tags, then resets the form', async ({
  page,
}) => {
  let payload: unknown
  await page.route('**/api/cards', async (route) => {
    payload = route.request().postDataJSON()
    await route.fulfill({ status: 201 })
  })
  await page.goto('/add-card')
  await expect(
    page.getByRole('link', { name: 'Add new card' }),
  ).toHaveAttribute('aria-current', 'page')
  await page.getByLabel('Question*:').fill('What is this?')
  await page.getByLabel('Answer:', { exact: true }).fill('An answer')
  await page
    .getByLabel('Tags (comma separated):')
    .fill(' rust, , svelte, rust ')
  await page.getByLabel('Question image').setInputFiles({
    name: 'question.png',
    mimeType: 'image/png',
    buffer: Buffer.from('fixture'),
  })

  await page.getByRole('button', { name: 'Create Flashcard' }).click()

  await expect(page.getByRole('status')).toHaveText('Card added successfully!')
  expect(payload).toEqual({
    question: 'What is this?',
    answer: 'An answer',
    examples: null,
    source: null,
    tags: ['rust', 'svelte'],
    question_img_fname: 'question.png',
    answer_img_fname: null,
  })
  await expect(page.getByLabel('Question*:')).toHaveValue('')
  await expect(page.getByLabel('Question image')).toHaveValue('')
})

test('preserves query prefills and entered data when creation fails', async ({
  page,
}) => {
  await page.route('**/api/cards', (route) => route.fulfill({ status: 500 }))
  await page.goto(
    '/add-card?question=Hola%3F&answer=Hello&source=story&tag=spanish',
  )
  await expect(page.getByLabel('Question*:')).toHaveValue('Hola?')
  await expect(page.getByLabel('Answer:', { exact: true })).toHaveValue('Hello')
  await expect(page.getByLabel('Source:')).toHaveValue('story')
  await expect(page.getByLabel('Tags (comma separated):')).toHaveValue(
    'spanish',
  )

  await page.getByRole('button', { name: 'Create Flashcard' }).click()

  await expect(page.getByRole('alert')).toContainText('HTTP 500')
  await expect(page.getByLabel('Question*:')).toHaveValue('Hola?')
  await expect(
    page.getByRole('button', { name: 'Create Flashcard' }),
  ).toBeEnabled()
})
