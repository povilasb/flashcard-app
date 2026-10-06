/** Shared class merging and element references for shadcn components. */
import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null
}

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
