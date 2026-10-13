import { expect, test } from '@playwright/test'

const card = {
  id: 7,
  question: 'Question',
  answer: 'Answer',
  examples: 'Example',
  img: null,
  question_img: null,
  tags: ['spanish'],
  last_reviewed: '2026-10-13T09:00:00Z',
}

test.beforeEach(async ({ page }) => {
  await page.route('**/api/rich-text/render', async (route) => {
    await route.fulfill({ json: { html: route.request().postDataJSON().text } })
  })
})

test('single-card deep link reveals content without submitting a review', async ({
  page,
}) => {
  await page.route('**/api/cards/7', (route) => route.fulfill({ json: card }))
  const writes: string[] = []
  page.on('request', (request) => {
    if (request.url().endsWith('/review')) writes.push(request.method())
  })
  await page.goto('/cards/7')

  await page.getByRole('button', { name: 'Show Answer' }).click()

  await expect(page.getByText('Answer', { exact: true })).toBeVisible()
  await expect(
    page.getByRole('button', { name: 'Remember', exact: true }),
  ).toHaveCount(0)
  await expect(
    page.getByRole('link', { name: 'Edit card', exact: true }),
  ).toHaveAttribute('href', '/cards/edit/7')
  expect(writes).toEqual([])
})

test('tag deep link loads all tagged cards and records a review', async ({
  page,
}) => {
  let requestedTag: string | null = null
  let review: unknown
  await page.route('**/api/cards?*', (route) => {
    requestedTag = new URL(route.request().url()).searchParams.get('tag')
    return route.fulfill({ json: [card] })
  })
  await page.route('**/api/cards/7/review', (route) => {
    review = route.request().postDataJSON()
    return route.fulfill({ status: 204 })
  })
  await page.goto('/review-cards/chapitre%203.caf%C3%A9')

  await page.getByRole('button', { name: 'Show Answer' }).click()
  await page.getByRole('button', { name: 'Remember', exact: true }).click()

  await expect(page.getByText('Nothing to review at the moment.')).toBeVisible()
  expect(requestedTag).toBe('chapitre 3.café')
  expect(review).toEqual({ remembered: true })
})
