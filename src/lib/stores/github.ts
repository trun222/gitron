import { writable, derived } from 'svelte/store';
import type { GitHubAuthInfo, DeviceCodeResponse } from '$lib/api/types';
import * as githubApi from '$lib/api/github';
import { openUrl } from '@tauri-apps/plugin-opener';

// Core state
export const authInfo = writable<GitHubAuthInfo | null>(null);
export const authLoading = writable(false);
export const authError = writable<string | null>(null);
export const deviceFlow = writable<DeviceCodeResponse | null>(null);
export const loginDialogOpen = writable(false);

// Derived
export const isAuthenticated = derived(authInfo, ($info) =>
  $info?.status.type === 'Authenticated'
);
export const currentUser = derived(authInfo, ($info) =>
  $info?.status.type === 'Authenticated' ? $info.status.user : null
);

/// Check stored auth on app startup
export async function initAuth() {
  authLoading.set(true);
  authError.set(null);
  try {
    const info = await githubApi.checkAuth();
    authInfo.set(info);
  } catch (e) {
    // Silently ignore — user just isn't logged in
    authInfo.set({ status: { type: 'NotAuthenticated' } });
  } finally {
    authLoading.set(false);
  }
}

/// Start the Device Flow login process
export async function startLogin() {
  authLoading.set(true);
  authError.set(null);

  try {
    // Request a device code from GitHub
    const dc = await githubApi.startDeviceFlow();
    deviceFlow.set(dc);
    loginDialogOpen.set(true);

    // Open the verification URL in the user's browser
    await openUrl(dc.verification_uri);

    // Poll for token (blocks until authorized or expired)
    const info = await githubApi.pollDeviceFlow(
      dc.device_code,
      dc.interval,
      dc.expires_in
    );
    authInfo.set(info);
    deviceFlow.set(null);
    loginDialogOpen.set(false);
  } catch (e) {
    authError.set(String(e));
    deviceFlow.set(null);
    loginDialogOpen.set(false);
  } finally {
    authLoading.set(false);
  }
}

/// Log out and clear state
export async function logout() {
  try {
    await githubApi.logout();
  } catch {
    // Best-effort
  }
  authInfo.set({ status: { type: 'NotAuthenticated' } });
  authError.set(null);
  deviceFlow.set(null);
}

/// Cancel an in-progress login
export function cancelLogin() {
  loginDialogOpen.set(false);
  deviceFlow.set(null);
  authLoading.set(false);
}
