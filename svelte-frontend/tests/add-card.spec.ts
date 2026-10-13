import { expect, test } from '@playwright/test'

const png = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jRZkAAAAASUVORK5CYII=',
  'base64',
)

test('text boxes attach pasted images to the matching field and preserve text paste', async ({
  page,
}) => {
  await page.goto('/add-card')
  await page.getByLabel('Question', { exact: true }).fill('Question text')
  await page.getByLabel('Answer', { exact: true }).fill('Answer text')

  for (const label of ['Question', 'Answer']) {
    await page.getByLabel(label, { exact: true }).evaluate(
      (element, bytes) => {
        const data = new DataTransfer()
        data.items.add(
          new File([new Uint8Array(bytes)], 'pasted.png', {
            type: 'image/png',
          }),
        )
        element.dispatchEvent(
          new ClipboardEvent('paste', {
            clipboardData: data,
            bubbles: true,
            cancelable: true,
          }),
        )
      },
      [...png],
    )
  }

  await expect(
    page.getByRole('img', { name: 'Question image preview' }),
  ).toBeVisible()
  await expect(
    page.getByRole('img', { name: 'Answer image preview' }),
  ).toBeVisible()
  await expect(page.getByLabel('Question', { exact: true })).toHaveValue(
    'Question text',
  )
  await expect(page.getByLabel('Answer', { exact: true })).toHaveValue(
    'Answer text',
  )

  const textPasteAllowed = await page
    .getByLabel('Question', { exact: true })
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

test('image picker opens, panel accepts drops, and invalid replacements preserve the image', async ({
  page,
}) => {
  await page.goto('/add-card')
  const control = page.getByRole('button', { name: 'Add question image' })

  const chooser = page.waitForEvent('filechooser')
  await control.click()

  expect((await chooser).isMultiple()).toBe(false)

  await page.getByRole('region', { name: 'Question panel' }).evaluate(
    (element, bytes) => {
      const data = new DataTransfer()
      data.items.add(
        new File([new Uint8Array(bytes)], 'dropped.png', { type: 'image/png' }),
      )
      element.dispatchEvent(
        new DragEvent('dragover', {
          dataTransfer: data,
          bubbles: true,
          cancelable: true,
        }),
      )
      element.dispatchEvent(
        new DragEvent('drop', {
          dataTransfer: data,
          bubbles: true,
          cancelable: true,
        }),
      )
    },
    [...png],
  )

  await expect(
    page.getByRole('img', { name: 'Question image preview' }),
  ).toBeVisible()
  await expect(control).toHaveText('Replace image')

  await page.getByLabel('Question image', { exact: true }).setInputFiles({
    name: 'bad.svg',
    mimeType: 'image/svg+xml',
    buffer: Buffer.from('<svg/>'),
  })

  await expect(page.getByRole('alert')).toHaveText(
    'Choose a PNG, JPEG, GIF, or WebP image.',
  )
  await expect(
    page.getByRole('img', { name: 'Question image preview' }),
  ).toBeVisible()

  await page.getByRole('button', { name: 'Remove question image' }).click()

  await expect(
    page.getByRole('img', { name: 'Question image preview' }),
  ).toHaveCount(0)
  await expect(page.getByRole('alert')).toHaveCount(0)
})

test('uploads an image and submits normalized tags, retaining details after success', async ({
  page,
}) => {
  let payload: unknown
  await page.route('**/api/media', async (route) => {
    expect(route.request().headers()['content-type']).toContain(
      'multipart/form-data',
    )
    expect(route.request().postDataBuffer()?.includes(png)).toBe(true)
    await route.fulfill({ status: 201, json: { filename: 'uploaded.png' } })
  })
  await page.route('**/api/cards', async (route) => {
    payload = route.request().postDataJSON()
    await route.fulfill({ status: 201 })
  })
  await page.goto('/add-card')
  await page.getByLabel('Question', { exact: true }).fill('What is this?')
  await page.getByLabel('Answer', { exact: true }).fill('An answer')
  await page.getByLabel('Source', { exact: true }).fill('Book')
  await page.getByLabel('Tags', { exact: true }).fill(' rust, , svelte, RUST ')
  await page.getByLabel('Tags', { exact: true }).press('Enter')
  await page
    .getByLabel('Question image', { exact: true })
    .setInputFiles({ name: 'question.png', mimeType: 'image/png', buffer: png })

  await page.getByRole('button', { name: 'Create card', exact: true }).click()

  await expect(page.getByRole('status')).toHaveText('Card added')
  expect(payload).toEqual({
    question: 'What is this?',
    answer: 'An answer',
    examples: null,
    source: 'Book',
    tags: ['rust', 'svelte'],
    question_img_fname: 'uploaded.png',
    answer_img_fname: null,
  })
  await expect(page.getByLabel('Question', { exact: true })).toHaveValue('')
  await expect(page.getByLabel('Question', { exact: true })).toBeFocused()
  await expect(page.getByLabel('Source', { exact: true })).toHaveValue('Book')
  await expect(
    page.getByRole('button', { name: 'Remove tag rust', exact: true }),
  ).toBeVisible()
  await expect(
    page.getByRole('button', { name: 'Remove question image' }),
  ).toHaveCount(0)
  await expect(page.getByText('1 card added this session')).toBeVisible()
})

test('preserves query prefills and all input when creation fails', async ({
  page,
}) => {
  await page.route('**/api/cards', (route) => route.fulfill({ status: 500 }))
  await page.goto(
    '/add-card?question=Hola%3F&answer=Hello&source=story&tag=spanish,Spanish,words',
  )
  await page.getByLabel('Example 1', { exact: true }).fill('Example')

  await page.getByRole('button', { name: 'Create card', exact: true }).click()

  await expect(page.getByRole('alert')).toContainText('HTTP 500')
  await expect(page.getByLabel('Question', { exact: true })).toHaveValue(
    'Hola?',
  )
  await expect(page.getByLabel('Answer', { exact: true })).toHaveValue('Hello')
  await expect(page.getByLabel('Source', { exact: true })).toHaveValue('story')
  await expect(
    page.getByRole('button', { name: 'Remove tag spanish', exact: true }),
  ).toHaveCount(1)
  await expect(page.getByLabel('Example 1', { exact: true })).toHaveValue(
    'Example',
  )
  await expect(
    page.getByRole('button', { name: 'Create card', exact: true }),
  ).toBeEnabled()
})

test('upload failure preserves the image and prevents card creation', async ({
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
  await page.getByLabel('Question', { exact: true }).fill('Clipboard question')
  await page
    .getByLabel('Answer image', { exact: true })
    .setInputFiles({ name: 'answer.png', mimeType: 'image/png', buffer: png })

  await page.getByRole('button', { name: 'Create card', exact: true }).click()

  await expect(page.getByRole('alert')).toContainText('Invalid image')
  await expect(
    page.getByRole('img', { name: 'Answer image preview' }),
  ).toBeVisible()
  await expect(page.getByLabel('Question', { exact: true })).toHaveValue(
    'Clipboard question',
  )
  expect(cardsPosted).toBe(false)
})

for (const modifier of ['Meta', 'Control']) {
  test(`${modifier}+Enter commits pending tags, joins examples, and clears details when unchecked`, async ({
    page,
  }) => {
    let payload: { examples: string; tags: string[] } | undefined
    await page.route('**/api/cards', async (route) => {
      payload = route.request().postDataJSON()
      await route.fulfill({ status: 201 })
    })
    await page.goto('/add-card')
    const create = page.getByRole('button', {
      name: 'Create card',
      exact: true,
    })
    await expect(create).toBeDisabled()
    await page.getByLabel('Question', { exact: true }).fill('   ')
    await expect(create).toBeDisabled()
    await page.getByLabel('Question', { exact: true }).fill('Question')
    await page.getByLabel('Example 1', { exact: true }).fill(' one ')
    await page.getByLabel('Example 1', { exact: true }).press('Enter')
    await expect(page.getByLabel('Example 2', { exact: true })).toBeFocused()
    await page.getByLabel('Example 2', { exact: true }).fill('two')
    await page.getByLabel('Source', { exact: true }).fill('Source')
    await page.getByLabel('Keep source & tags for the next card').uncheck()
    await page.getByLabel('Tags', { exact: true }).fill('pending')

    await page.getByLabel('Tags', { exact: true }).press(`${modifier}+Enter`)

    await expect(page.getByRole('status')).toHaveText('Card added')
    expect(payload?.examples).toBe('one\ntwo')
    expect(payload?.tags).toEqual(['pending'])
    await expect(page.getByLabel('Source', { exact: true })).toHaveValue('')
    await expect(
      page.getByRole('button', { name: 'Remove tag pending' }),
    ).toHaveCount(0)
    await expect(page.getByLabel('Example 1', { exact: true })).toHaveValue('')
    await expect(page.getByLabel('Example 2', { exact: true })).toHaveCount(0)
  })
}

test('preview flips without network calls; tags, examples and Clear are editable', async ({
  page,
}) => {
  const calls: string[] = []
  page.on('request', (request) => {
    if (request.url().includes('/api/')) calls.push(request.url())
  })
  await page.goto(
    '/add-card?question=Front&answer=Back&source=Book&tag=one,two',
  )
  const preview = page.getByRole('button', { name: 'Flip preview card' })
  await expect(preview).toContainText('Front')
  await page.getByLabel('Example 1', { exact: true }).fill('Usage')

  await preview.click()

  await expect(preview).toContainText('Back')
  await expect(preview).toContainText('Usage')
  await expect(
    page.getByRole('button', { name: 'Back', exact: true }),
  ).toHaveAttribute('aria-pressed', 'true')

  await page.getByLabel('Tags', { exact: true }).press('Backspace')
  await page.getByRole('button', { name: 'Remove example 1' }).click()

  await expect(
    page.getByRole('button', { name: 'Remove tag two' }),
  ).toHaveCount(0)
  await expect(page.getByLabel('Example 1', { exact: true })).toHaveValue('')

  await page.getByRole('button', { name: 'Clear', exact: true }).click()

  await expect(page.getByLabel('Question', { exact: true })).toHaveValue('')
  await expect(page.getByLabel('Source', { exact: true })).toHaveValue('')
  await expect(
    page.getByRole('button', { name: 'Remove tag one' }),
  ).toHaveCount(0)
  await expect(preview).toContainText('Your question will appear here')
  expect(calls).toEqual([])
})

test('submission disables editing and prevents duplicate requests', async ({
  page,
}) => {
  let finish!: () => void
  const pending = new Promise<void>((resolve) => (finish = resolve))
  let requests = 0
  await page.route('**/api/cards', async (route) => {
    requests += 1
    await pending
    await route.fulfill({ status: 201 })
  })
  await page.goto('/add-card?question=Question')

  await page.getByRole('button', { name: 'Create card', exact: true }).click()

  await expect(
    page.getByRole('button', { name: 'Creating…', exact: true }),
  ).toBeDisabled()
  await expect(page.getByLabel('Question', { exact: true })).toBeDisabled()
  await expect(page.locator('form')).toHaveAttribute('aria-busy', 'true')
  expect(requests).toBe(1)

  finish()

  await expect(page.getByRole('status')).toHaveText('Card added')
})

test('narrow layout wraps the preview below the form without horizontal overflow', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await page.goto('/add-card')

  const form = await page.locator('form').boundingBox()
  const preview = await page
    .getByRole('complementary', { name: 'Card preview' })
    .boundingBox()
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth > innerWidth,
  )

  expect(preview!.y).toBeGreaterThan(form!.y + form!.height)
  expect(overflow).toBe(false)
})
