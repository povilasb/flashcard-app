import { expect, test } from '@playwright/test'

const card = {
  id: 7,
  question: 'Original question',
  answer: 'Original answer',
  examples: 'First example\nSecond example',
  source: 'Book',
  tags: ['rust'],
  question_img: 'question.png',
  img: 'answer.png',
  last_reviewed: '2026-01-01T12:00:00Z',
  review_after_secs: 86400,
}

test('list link opens Svelte editing and saves without changing existing images', async ({
  page,
}) => {
  let saved: unknown
  await page.route('**/api/cards', (route) => route.fulfill({ json: [card] }))
  await page.route('**/api/cards/7', async (route) => {
    if (route.request().method() === 'PUT') {
      saved = route.request().postDataJSON()
      await route.fulfill({ status: 204 })
    } else await route.fulfill({ json: card })
  })
  await page.goto('/list-cards')

  await page
    .getByRole('link', { name: 'Original question', exact: true })
    .click()

  await expect(page).toHaveURL(/\/cards\/edit\/7$/)
  await expect(page.getByRole('heading', { name: 'Edit card' })).toBeVisible()
  await expect(page.getByLabel('Question', { exact: true })).toHaveValue(
    card.question,
  )
  await expect(page.getByLabel('Example 2', { exact: true })).toHaveValue(
    'Second example',
  )
  await expect(
    page.getByRole('img', { name: 'Question image preview' }),
  ).toHaveAttribute('src', '/media/question.png')
  await expect(
    page.getByRole('button', { name: 'Flip preview card' }),
  ).toContainText(card.question)
  await expect(
    page.getByLabel('Keep source & tags for the next card'),
  ).toHaveCount(0)

  await page.getByLabel('Question', { exact: true }).fill('Edited question')
  await page.getByRole('button', { name: 'Save changes', exact: true }).click()

  await expect(page.getByRole('status')).toHaveText('Card saved')
  await expect(page.getByLabel('Question', { exact: true })).toHaveValue(
    'Edited question',
  )
  expect(saved).toEqual({
    question: 'Edited question',
    answer: card.answer,
    examples: card.examples,
    source: card.source,
    tags: card.tags,
    question_img_fname: card.question_img,
    answer_img_fname: card.img,
  })
})

test('replaces and removes images, preserving changes when save fails', async ({
  page,
}) => {
  let saved: Record<string, unknown> | undefined
  let uploads = 0
  await page.route('**/api/cards/7', async (route) => {
    if (route.request().method() === 'PUT') {
      saved = route.request().postDataJSON()
      await route.fulfill({ status: 500 })
    } else await route.fulfill({ json: card })
  })
  await page.route('**/api/media', (route) => {
    uploads++
    return route.fulfill({ status: 201, json: { filename: 'replacement.png' } })
  })
  await page.goto('/cards/edit/7')
  await page.getByRole('button', { name: 'Remove question image' }).click()
  await page.getByLabel('Answer image', { exact: true }).setInputFiles({
    name: 'new.png',
    mimeType: 'image/png',
    buffer: Buffer.from('fixture'),
  })
  await page.getByLabel('Answer', { exact: true }).fill('Changed answer')

  await page.getByRole('button', { name: 'Save changes', exact: true }).click()

  await expect(page.getByRole('alert')).toContainText('HTTP 500')
  expect(saved?.question_img_fname).toBeNull()
  expect(saved?.answer_img_fname).toBe('replacement.png')
  expect(uploads).toBe(1)
  await expect(
    page.getByRole('img', { name: 'Question image preview' }),
  ).toHaveCount(0)
  await expect(page.getByLabel('Answer', { exact: true })).toHaveValue(
    'Changed answer',
  )
  await expect(
    page.getByRole('img', { name: 'Answer image preview' }),
  ).toHaveAttribute('src', /^blob:/)
  await expect(
    page.getByRole('button', { name: 'Save changes', exact: true }),
  ).toBeEnabled()
})

test('missing cards show an error rather than an empty editor', async ({
  page,
}) => {
  await page.route('**/api/cards/99', (route) => route.fulfill({ status: 404 }))

  await page.goto('/cards/edit/99')

  await expect(page.getByRole('alert')).toHaveText('Card not found.')
  await expect(page.getByRole('button', { name: 'Save changes' })).toHaveCount(
    0,
  )
  await expect(
    page.getByRole('link', { name: 'Back to cards' }),
  ).toHaveAttribute('href', '/list-cards')
})
