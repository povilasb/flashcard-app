<script lang="ts">
  import { Image, X } from 'lucide-svelte'

  let {
    label,
    file = $bindable<File | null>(null),
    disabled = false,
    preview = $bindable(''),
    filename = $bindable<string | null>(null),
  }: {
    label: string
    file?: File | null
    disabled?: boolean
    preview?: string
    filename?: string | null
  } = $props()

  let error = $state<string | null>(null)
  let input: HTMLInputElement

  $effect(() => {
    const url = file ? URL.createObjectURL(file) : ''
    preview =
      url ||
      (filename
        ? `/media/${filename.split('/').map(encodeURIComponent).join('/')}`
        : '')
    if (!file && input) input.value = ''
    return () => {
      if (url) URL.revokeObjectURL(url)
    }
  })

  export function selectFile(selected: File | undefined) {
    if (!selected || disabled) return
    error = null
    if (
      !['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(
        selected.type,
      )
    ) {
      error = 'Choose a PNG, JPEG, GIF, or WebP image.'
    } else if (selected.size > 10 * 1024 * 1024) {
      error = 'Image must be 10 MiB or smaller.'
    } else {
      file = selected
    }
    input.value = ''
  }

  export function paste(event: ClipboardEvent) {
    const pasted = Array.from(event.clipboardData?.files ?? [])[0]
    if (!pasted || disabled) return
    event.preventDefault()
    selectFile(pasted)
  }

  export function clear() {
    file = null
    filename = null
    error = null
    if (input) input.value = ''
  }
</script>

<div class="image-row">
  {#if preview}
    <div class="thumb">
      <img src={preview} alt={`${label} preview`} />
      <button
        type="button"
        class="thumb-x"
        {disabled}
        aria-label={`Remove ${label.toLowerCase()}`}
        onclick={clear}><X size={14} aria-hidden="true" /></button
      >
    </div>
  {/if}
  <label for={label.replaceAll(' ', '-')} class="sr-only">{label}</label>
  <input
    id={label.replaceAll(' ', '-')}
    bind:this={input}
    type="file"
    hidden
    accept="image/png,image/jpeg,image/gif,image/webp"
    aria-label={label}
    {disabled}
    onchange={(event) => selectFile(event.currentTarget.files?.[0])}
  />
  <button
    type="button"
    class="image-picker"
    aria-label={`Add ${label.toLowerCase()}`}
    {disabled}
    onclick={() => input.click()}
    onpaste={paste}
    ondragover={(event) => event.preventDefault()}
    ondrop={(event) => {
      event.preventDefault()
      event.stopPropagation()
      selectFile(event.dataTransfer?.files[0])
    }}
  >
    <Image size={18} aria-hidden="true" />{preview
      ? 'Replace image'
      : 'Add image'}
  </button>
  <span class="image-hint">or drop / paste an image</span>
</div>
{#if error}<p role="alert" class="image-error">{error}</p>{/if}

<style>
  .image-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
  }
  .thumb {
    position: relative;
    width: 112px;
    height: 80px;
    border-radius: 10px;
    overflow: hidden;
    border: 1px solid var(--color-line);
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .thumb-x {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 28px;
    height: 28px;
    border-radius: 999px;
    background: rgb(21 23 28 / 0.72);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }
  button:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }
  button:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .image-error {
    font-size: 13px;
    color: var(--color-danger);
  }
  .image-picker {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 0 14px;
    min-height: 44px;
    border-radius: 10px;
    font-size: 14px;
    font-weight: 600;
    text-align: left;
    cursor: pointer;
    color: var(--color-ink-2);
    transition: background 0.15s;
    &:hover {
      background: var(--color-chip);
    }
  }

  .image-hint {
    color: var(--color-hint);
    font-size: 13px;
  }
</style>
