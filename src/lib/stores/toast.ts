import { writable } from 'svelte/store';

export interface Toast {
  id: number;
  message: string;
  type: 'success' | 'error' | 'info';
}

let nextId = 0;

export const toasts = writable<Toast[]>([]);

const DEFAULT_DURATION = 4000;

export function addToast(
  message: string,
  type: Toast['type'] = 'info',
  duration: number = DEFAULT_DURATION
) {
  const id = nextId++;
  toasts.update((t) => [...t, { id, message, type }]);
  setTimeout(() => dismissToast(id), duration);
}

export function dismissToast(id: number) {
  toasts.update((t) => t.filter((toast) => toast.id !== id));
}
