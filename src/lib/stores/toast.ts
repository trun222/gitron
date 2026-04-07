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
): number {
  const id = nextId++;
  toasts.update((t) => [...t, { id, message, type }]);
  if (duration > 0) {
    setTimeout(() => dismissToast(id), duration);
  }
  return id;
}

export function updateToast(
  id: number,
  message: string,
  type?: Toast['type'],
  duration?: number,
) {
  toasts.update((t) =>
    t.map((toast) =>
      toast.id === id
        ? { ...toast, message, type: type ?? toast.type }
        : toast
    )
  );
  if (duration !== undefined && duration > 0) {
    setTimeout(() => dismissToast(id), duration);
  }
}

export function dismissToast(id: number) {
  toasts.update((t) => t.filter((toast) => toast.id !== id));
}
