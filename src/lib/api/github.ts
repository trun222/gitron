import { getTransport } from '$lib/api';
import type { GitHubAuthInfo, DeviceCodeResponse, GitHubUser, GitHubRepo } from './types';

export async function checkAuth(): Promise<GitHubAuthInfo> {
  return getTransport().invoke('github_check_auth');
}

export async function startDeviceFlow(): Promise<DeviceCodeResponse> {
  return getTransport().invoke('github_start_device_flow');
}

export async function pollDeviceFlow(
  deviceCode: string,
  interval: number,
  expiresIn: number
): Promise<GitHubAuthInfo> {
  return getTransport().invoke('github_poll_device_flow', { deviceCode, interval, expiresIn });
}

export async function logout(): Promise<void> {
  return getTransport().invoke('github_logout');
}

export async function getUser(): Promise<GitHubUser | null> {
  return getTransport().invoke('github_get_user');
}

export async function listRepos(): Promise<GitHubRepo[]> {
  return getTransport().invoke('github_list_repos');
}
