import { expect, test } from '@playwright/test'

test('text boxes attach pasted images to the matching field and preserve text paste', async ({
  page,
}) => {
  await page.goto('/add-card')
  await page.getByLabel('Question*:').fill('Question text')
  await page.getByLabel('Answer:', { exact: true }).fill('Answer text')

  for (const label of ['Question*:', 'Answer:']) {
    await page.getByLabel(label, { exact: true }).evaluate((element) => {
      const data = new DataTransfer()
      data.items.add(new File(['fixture'], 'pasted.png', { type: 'image/png' }))
      element.dispatchEvent(
        new ClipboardEvent('paste', {
          clipboardData: data,
          bubbles: true,
          cancelable: true,
        }),
      )
    })
  }

  await expect(
    page.getByRole('img', { name: 'Question image preview' }),
  ).toBeVisible()
  await expect(
    page.getByRole('img', { name: 'Answer image preview' }),
  ).toBeVisible()
  await expect(page.getByLabel('Question*:')).toHaveValue('Question text')
  await expect(page.getByLabel('Answer:', { exact: true })).toHaveValue(
    'Answer text',
  )

  const textPasteAllowed = await page
    .getByLabel('Question*:')
    .evaluate((element) => {
      const data = new DataTransfer()
      data.setData('text/plain', 'ordinary text')
      return element.dispatchEvent(
        new ClipboardEvent('paste', {
          clipboardData: data,
          bubbles: true,
          cancelable: true,
        }),
      )
    })

  expect(textPasteAllowed).toBe(true)
})

test('compact image control opens the picker and accepts dropped files', async ({
  page,
}) => {
  await page.goto('/add-card')
  const control = page.getByRole('button', { name: 'Add question image' })

  const chooser = page.waitForEvent('filechooser')
  await control.click()

  expect((await chooser).isMultiple()).toBe(false)

  await control.evaluate((element) => {
    const data = new DataTransfer()
    data.items.add(new File(['fixture'], 'dropped.png', { type: 'image/png' }))
    element.dispatchEvent(
      new DragEvent('drop', {
        dataTransfer: data,
        bubbles: true,
        cancelable: true,
      }),
    )
  })

  await expect(
    page.getByRole('img', { name: 'Question image preview' }),
  ).toBeVisible()
})

test('creates a card with filenames and normalized tags, then resets the form', async ({
  page,
}) => {
  let payload: unknown
  await page.route('**/api/media', async (route) => {
    expect(route.request().headers()['content-type']).toContain(
      'multipart/form-data',
    )
    expect(route.request().postDataBuffer()?.toString()).toContain('fixture')
    await route.fulfill({ status: 201, json: { filename: 'uploaded.png' } })
  })
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
  await page.getByLabel('Question image', { exact: true }).setInputFiles({
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
    question_img_fname: 'uploaded.png',
    answer_img_fname: null,
  })
  await expect(page.getByLabel('Question*:')).toHaveValue('')
  await expect(page.getByLabel('Question image', { exact: true })).toHaveValue(
    '',
  )
  await expect(
    page.getByRole('button', { name: 'Remove question image' }),
  ).toHaveCount(0)
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

test('pastes and removes an image, then retains a replacement when upload fails', async ({
  page,
}) => {
  let cardsPosted = false
  await page.route('**/api/media', (route) =>
    route.fulfill({ status: 422, body: 'Invalid image' }),
  )
  await page.route('**/api/cards', (route) => {
    cardsPosted = true
    return route.fulfill({ status: 201 })
  })
  await page.goto('/add-card')
  await page.getByLabel('Question*:').fill('Clipboard question')

  await page
    .getByRole('button', { name: 'Add answer image' })
    .evaluate((element) => {
      const clipboard = new DataTransfer()
      clipboard.items.add(
        new File(['fixture'], 'pasted.png', { type: 'image/png' }),
      )
      element.dispatchEvent(
        new ClipboardEvent('paste', {
          clipboardData: clipboard,
          bubbles: true,
        }),
      )
    })

  await expect(
    page.getByRole('img', { name: 'Answer image preview' }),
  ).toBeVisible()

  await page.getByRole('button', { name: 'Remove answer image' }).click()

  await expect(
    page.getByRole('img', { name: 'Answer image preview' }),
  ).toHaveCount(0)

  await page.getByLabel('Answer image', { exact: true }).setInputFiles({
    name: 'replacement.png',
    mimeType: 'image/png',
    buffer: Buffer.from('fixture'),
  })
  await page.getByRole('button', { name: 'Create Flashcard' }).click()

  await expect(page.getByRole('alert')).toContainText('Invalid image')
  await expect(
    page.getByRole('img', { name: 'Answer image preview' }),
  ).toBeVisible()
  await expect(page.getByLabel('Question*:')).toHaveValue('Clipboard question')
  expect(cardsPosted).toBe(false)
})
