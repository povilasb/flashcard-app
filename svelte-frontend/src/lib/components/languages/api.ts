/** HTTP and flashcard handoff shared by the language-learning pages. */
export async function request<T>(
  path: string,
  body?: unknown,
  method = body === undefined ? 'GET' : 'POST',
): Promise<T> {
  const response = await fetch(`/api/languages/${path}`, {
    method,
    ...(body === undefined
      ? {}
      : {
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(body),
        }),
  })
  if (!response.ok) {
    const text = await response.text()
    let message = `Request failed (HTTP ${response.status})`
    try {
      message = JSON.parse(text).error?.message || message
    } catch {
      /* Non-JSON proxy errors use the HTTP status. */
    }
    throw new Error(message)
  }
  return response.status === 204 || response.status === 201
    ? (undefined as T)
    : response.json()
}

export function flashcardLink(
  answer: string,
  language: string,
  question?: string,
) {
  const params = new URLSearchParams({
    answer,
    tag: language,
    source: 'learning-languages app',
  })
  if (question !== undefined) params.set('question', question)
  return `/add-card?${params}`
}
