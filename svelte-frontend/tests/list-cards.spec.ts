import { expect, test } from '@playwright/test'

const cards = [
  {
    id: 1,
    question: 'Alpha',
    answer: 'Hello',
    tags: ['spanish'],
    last_reviewed: '2026-10-13T09:00:00Z',
  },
  {
    id: 2,
    question: 'Beta',
    answer: 'Ownership',
    tags: ['rust'],
    last_reviewed: '2026-01-01T09:00:00Z',
  },
  {
    id: 3,
    question: 'Gamma',
    answer: 'Goodbye',
    tags: ['spanish'],
    last_reviewed: '2026-10-12T09:00:00Z',
  },
]

test.beforeEach(async ({ page }) => {
  await page.clock.install({ time: new Date('2026-10-13T12:00:00Z') })
  await page.clock.pauseAt(new Date('2026-10-13T12:00:01Z'))
  await page.route('**/api/cards', (route) => route.fulfill({ json: cards }))
})

test('counts, search, tag filters, sorting, edit links and review age', async ({
  page,
}) => {
  await page.goto('/list-cards')
  const rows = page.locator('tbody tr')
  const search = page.getByRole('searchbox', { name: 'Search cards' })
  const filters = page.getByRole('group', { name: 'Filter by tag' })

  await expect(page.getByLabel('Card count')).toHaveText('3')
  await expect(page.locator('footer')).toContainText('3 cards')
  await expect(rows.locator('td:first-child')).toHaveText([
    'Alpha',
    'Gamma',
    'Beta',
  ])
  await expect(
    page.getByRole('link', { name: 'Edit card: Beta', exact: true }),
  ).toHaveAttribute('href', '/cards/edit/2')
  await expect(rows.filter({ hasText: 'Beta' }).locator('.stale')).toHaveText(
    '9 months ago',
  )

  for (const [query, count] of [
    ['ALPHA', 1],
    ['ownership', 1],
    ['spanish', 2],
  ] as const) {
    await search.fill(query)

    await expect(rows).toHaveCount(count)
    await expect(page.locator('footer')).toContainText(
      `Showing ${count} of 3 cards`,
    )
  }

  await search.press('Escape')
  await filters.getByRole('button', { name: 'rust 1' }).click()

  await expect(filters.getByRole('button', { name: 'rust 1' })).toHaveAttribute(
    'aria-pressed',
    'true',
  )
  await expect(rows).toHaveCount(1)

  await filters.getByRole('button', { name: 'rust 1' }).click()
  await page
    .getByRole('button', { name: 'Filter by tag spanish', exact: true })
    .first()
    .click()

  await expect(rows).toHaveCount(2)

  await filters.getByRole('button', { name: 'All 3' }).click()
  await page.getByLabel('Sort', { exact: true }).selectOption('oldest')

  await expect(rows.locator('td:first-child')).toHaveText([
    'Beta',
    'Gamma',
    'Alpha',
  ])

  await page.getByLabel('Sort', { exact: true }).selectOption('question')

  await expect(rows.locator('td:first-child')).toHaveText([
    'Alpha',
    'Beta',
    'Gamma',
  ])
})

test('stacked undo restores original order without sending deletes', async ({
  page,
}) => {
  const deleted: string[] = []
  await page.route('**/api/cards/*', (route) => {
    deleted.push(route.request().url())
    return route.fulfill({ status: 204 })
  })
  await page.goto('/list-cards')

  await page
    .getByRole('button', { name: 'Delete card: Alpha', exact: true })
    .click()

  await expect(page.getByRole('status')).toContainText('Deleted "Alpha"')
  await expect(page.getByLabel('Card count')).toHaveText('2')
  await expect(
    page.getByRole('button', { name: 'Delete card: Gamma', exact: true }),
  ).toBeFocused()

  await page
    .getByRole('button', { name: 'Delete card: Gamma', exact: true })
    .click()
  await page.getByRole('button', { name: 'Undo', exact: true }).click()
  await page.getByRole('button', { name: 'Undo', exact: true }).click()
  await page.clock.runFor(6000)

  await expect(page.locator('tbody tr td:first-child')).toHaveText([
    'Alpha',
    'Gamma',
    'Beta',
  ])
  expect(deleted).toEqual([])
})

test('delete waits five seconds; failure restores the card and reports the error', async ({
  page,
}) => {
  const deleted: string[] = []
  await page.route('**/api/cards/*', (route) => {
    deleted.push(route.request().method())
    return route.fulfill({ status: 500 })
  })
  await page.goto('/list-cards')
  await page
    .getByRole('button', { name: 'Delete card: Beta', exact: true })
    .click()

  await page.clock.runFor(4999)

  expect(deleted).toEqual([])
  await expect(page.locator('tbody tr')).toHaveCount(2)

  await page.clock.runFor(1)

  await expect(page.getByRole('alert')).toHaveText(
    "Couldn't delete card. HTTP 500",
  )
  await expect(page.locator('tbody tr')).toHaveCount(3)
  expect(deleted).toEqual(['DELETE'])
})

test('successful delete is committed after the undo window', async ({
  page,
}) => {
  const deleted: string[] = []
  await page.route('**/api/cards/*', (route) => {
    deleted.push(route.request().method())
    return route.fulfill({ status: 204 })
  })
  await page.goto('/list-cards')
  await page
    .getByRole('button', { name: 'Delete card: Alpha', exact: true })
    .click()

  await page.clock.runFor(5000)

  await expect.poll(() => deleted).toEqual(['DELETE'])
  await expect(page.locator('tbody tr')).toHaveCount(2)
  await expect(
    page.getByRole('button', { name: 'Undo', exact: true }),
  ).toHaveCount(0)
})

test('navigation flushes pending deletes with keepalive', async ({ page }) => {
  await page.goto('/list-cards')
  await page.evaluate(() => {
    const original = window.fetch
    window.fetch = (input, init) => {
      if (init?.method === 'DELETE')
        sessionStorage.setItem(
          'delete',
          JSON.stringify({ url: input, keepalive: init.keepalive }),
        )
      return original(input, init)
    }
  })
  await page.route('**/api/cards/1', (route) => route.fulfill({ status: 204 }))
  await page
    .getByRole('button', { name: 'Delete card: Alpha', exact: true })
    .click()

  await page.getByRole('link', { name: 'New card', exact: true }).click()

  await expect(page).toHaveURL(/\/add-card$/)
  expect(
    await page.evaluate(() => JSON.parse(sessionStorage.getItem('delete')!)),
  ).toEqual({ url: '/api/cards/1', keepalive: true })
})

test('search shortcut, empty results and mobile scrolling', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto('/list-cards')
  await expect(page.locator('tbody tr')).toHaveCount(3)

  await page.keyboard.press('/')

  await expect(page.getByRole('searchbox')).toBeFocused()
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth > innerWidth,
    ),
  ).toBe(false)
  expect(
    await page
      .locator('.scroll')
      .evaluate((el) => el.scrollWidth > el.clientWidth),
  ).toBe(true)

  await page.getByRole('searchbox').fill('not found')

  await expect(page.getByText('No cards match', { exact: true })).toBeVisible()

  await page.getByRole('button', { name: 'Clear filters' }).click()

  await expect(page.locator('tbody tr')).toHaveCount(3)
})

test('empty and failed loading states', async ({ page }) => {
  await page.route('**/api/cards', (route) => route.fulfill({ status: 500 }))

  await page.goto('/list-cards')

  await expect(page.getByRole('alert')).toContainText('HTTP 500')

  await page.route('**/api/cards', (route) => route.fulfill({ json: [] }))
  await page.getByRole('button', { name: 'Retry' }).click()

  await expect(page.getByText('No cards yet.', { exact: true })).toBeVisible()
})

test('shows the top five tags and toggles the rest', async ({ page }) => {
  await page.route('**/api/cards', (route) =>
    route.fulfill({
      json: [...cards, { ...cards[0], id: 4, tags: ['a', 'b', 'c', 'd', 'e'] }],
    }),
  )
  await page.goto('/list-cards')
  const filters = page.getByRole('group', { name: 'Filter by tag' })

  await expect(filters.locator('button[aria-pressed="false"]')).toHaveText([
    'spanish 2',
    'a 1',
    'b 1',
    'c 1',
    'd 1',
  ])
  await expect(
    filters.getByRole('button', { name: 'rust 1', exact: true }),
  ).toHaveCount(0)

  await filters
    .getByRole('button', { name: 'Show more (2)', exact: true })
    .click()

  await expect(
    filters.getByRole('button', { name: 'Show less' }),
  ).toHaveAttribute('aria-expanded', 'true')
  await expect(
    filters.getByRole('button', { name: 'rust 1', exact: true }),
  ).toBeVisible()

  await filters.getByRole('button', { name: 'rust 1', exact: true }).click()

  await expect(page.locator('tbody tr')).toHaveCount(1)

  await filters.getByRole('button', { name: 'Show less' }).click()

  await expect(
    filters.getByRole('button', { name: 'Show more (2)', exact: true }),
  ).toHaveAttribute('aria-expanded', 'false')
  await expect(
    filters.getByRole('button', { name: 'rust 1', exact: true }),
  ).toHaveCount(0)
})
