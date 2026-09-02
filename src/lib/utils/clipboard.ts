/**
 * Copy text to the clipboard.
 *
 * Tries the async Clipboard API first, then falls back to selecting an element
 * and running `document.execCommand('copy')`. The fallback matters on Linux,
 * where WebKitGTK (Tauri's webview) often rejects `navigator.clipboard.writeText`.
 *
 * Pass `source` when the text already lives in a textarea/input on screen; the
 * fallback then selects that element instead of a temporary one, which keeps
 * focus inside dialogs that trap it.
 */
export async function copyToClipboard(
  text: string,
  source?: HTMLTextAreaElement | HTMLInputElement | null
): Promise<boolean> {
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // fall through to the legacy path
  }

  try {
    const previouslyFocused = document.activeElement as HTMLElement | null;
    let el = source ?? null;
    const isTemp = !el;
    if (!el) {
      el = document.createElement('textarea');
      el.value = text;
      el.setAttribute('readonly', '');
      el.style.position = 'fixed';
      el.style.top = '0';
      el.style.left = '0';
      el.style.opacity = '0';
      document.body.appendChild(el);
    }
    el.focus();
    el.select();
    el.setSelectionRange(0, el.value.length);
    const ok = document.execCommand('copy');
    if (isTemp) {
      el.remove();
    } else {
      el.setSelectionRange(0, 0);
    }
    previouslyFocused?.focus?.();
    return ok;
  } catch {
    return false;
  }
}
