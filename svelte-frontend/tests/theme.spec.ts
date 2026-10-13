import { expect, test } from '@playwright/test'

test.beforeEach(async ({ page }) => {
  await page.route('**/api/cards', (route) => route.fulfill({ json: [] }))
  await page.route('**/api/cards/review', (route) =>
    route.fulfill({
      json: [
        {
          id: 1,
          question: 'Question',
          answer: 'Answer',
          tags: [],
          examples: null,
          img: null,
          question_img: null,
          last_reviewed: '2026-10-13T09:00:00Z',
        },
      ],
    }),
  )
  await page.route('**/api/languages/vocabulary', (route) =>
    route.fulfill({ json: [] }),
  )
  await page.route('**/api/rich-text/render', (route) =>
    route.fulfill({ json: { html: route.request().postDataJSON().text } }),
  )
})

test('theme toggle supports keyboard use and persists across pages and reloads', async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: 'light' })
  await page.goto('/list-cards')
  const toggle = page.getByRole('button', { name: 'Switch to dark theme' })

  await toggle.focus()
  await page.keyboard.press('Enter')

  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark')
  await expect(
    page.getByRole('button', { name: 'Switch to light theme' }),
  ).toBeVisible()
  await expect(page.locator('body')).toHaveCSS(
    'background-color',
    'rgb(20, 23, 30)',
  )

  await page.getByRole('link', { name: 'Add new card', exact: true }).click()
  await page.reload()

  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark')
  await expect(page.locator('textarea').first()).toHaveCSS(
    'background-color',
    'rgb(29, 34, 44)',
  )

  await page.getByRole('button', { name: 'Switch to light theme' }).click()
  await page.reload()

  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light')
  await expect(page.locator('textarea').first()).toHaveCSS(
    'background-color',
    'rgb(255, 255, 255)',
  )
})

test('system dark theme covers list, review, and language surfaces on mobile', async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.setViewportSize({ width: 390, height: 844 })

  await page.goto('/list-cards')

  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark')
  await expect(page.locator('.panel').first()).toHaveCSS(
    'background-color',
    'rgb(29, 34, 44)',
  )
  await expect(
    page.getByRole('button', { name: 'Switch to light theme' }),
  ).toBeInViewport()

  await page.goto('/review-cards')

  await expect(page.locator('.rounded-lg').first()).toHaveCSS(
    'background-color',
    'rgb(29, 34, 44)',
  )

  await page.goto('/learn-languages/vocabulary')

  await expect(page.getByLabel('Word', { exact: true })).toHaveCSS(
    'background-color',
    'rgb(29, 34, 44)',
  )
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark')
})

test('saved light theme overrides a dark system preference', async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.addInitScript(() => localStorage.setItem('theme', 'light'))

  await page.goto('/list-cards')

  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light')
  await expect(
    page.getByRole('button', { name: 'Switch to dark theme' }),
  ).toBeVisible()
})

test('theme toggle still works when browser storage is unavailable', async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: 'light' })
  await page.addInitScript(() => {
    Object.defineProperty(window, 'localStorage', {
      get() {
        throw new Error('Storage blocked')
      },
    })
  })
  await page.goto('/list-cards')

  await page.getByRole('button', { name: 'Switch to dark theme' }).click()

  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark')
})
