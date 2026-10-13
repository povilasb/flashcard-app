<script lang="ts">
  import { Image } from 'lucide-svelte'

  let {
    label,
    file = $bindable<File | null>(null),
    disabled = false,
  }: {
    label: string
    file?: File | null
    disabled?: boolean
  } = $props()

  let preview = $state('')
  let error = $state<string | null>(null)
  let input: HTMLInputElement

  $effect(() => {
    const url = file ? URL.createObjectURL(file) : ''
    preview = url
    if (!file && input) input.value = ''
    return () => {
      if (url) URL.revokeObjectURL(url)
    }
  })

  function selectFile(selected: File | undefined) {
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
</script>

<div class="ml-4 flex flex-col gap-2">
  <input
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
      selectFile(event.dataTransfer?.files[0])
    }}
  >
    <span class="image-action"><Image size={20} /> Add image</span>
    <span class="image-hint">or drop / paste into the text box</span>
  </button>
  {#if preview}
    <img
      src={preview}
      alt={`${label} preview`}
      class="max-h-48 max-w-full rounded object-contain"
    />
    <button
      type="button"
      {disabled}
      class="self-start text-sm text-red-600 hover:underline"
      onclick={() => {
        file = null
        error = null
      }}>Remove {label.toLowerCase()}</button
    >
  {/if}
  {#if error}<p role="alert" class="text-sm text-red-600">{error}</p>{/if}
</div>

<style>
  .image-picker {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.5rem 1.5rem;
    padding: 0.75rem 0;
    border-radius: 0.25rem;
    text-align: left;
    cursor: pointer;
    color: #374151;

    &:focus-visible {
      outline: 2px solid #3b82f6;
      outline-offset: 4px;
    }

    &:disabled {
      opacity: 0.5;
      cursor: default;
    }
  }

  .image-action {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
  }

  .image-hint {
    color: #6b7280;
  }
</style>
