import { writable, derived, get } from 'svelte/store';
import type { GitHubRepo } from '$lib/api/types';
import * as githubApi from '$lib/api/github';
import * as repoApi from '$lib/api/repo';
import { openRepo } from '$lib/stores/repo';
import { addOutput } from '$lib/stores/output';
import { isAuthenticated } from '$lib/stores/github';

// State
export const cloneDialogOpen = writable(false);
export const githubRepos = writable<GitHubRepo[]>([]);
export const githubReposLoading = writable(false);
export const cloning = writable(false);
export const cloneError = writable<string | null>(null);
export const repoSearchQuery = writable('');

// Derived
export const filteredRepos = derived(
  [githubRepos, repoSearchQuery],
  ([$repos, $query]) => {
    const q = $query.toLowerCase().trim();
    if (!q) return $repos;
    return $repos.filter(
      (r) =>
        r.full_name.toLowerCase().includes(q) ||
        (r.description?.toLowerCase().includes(q) ?? false)
    );
  }
);

// Actions
export function openCloneDialog() {
  cloneDialogOpen.set(true);
  cloneError.set(null);
  if (get(isAuthenticated) && get(githubRepos).length === 0) {
    loadGitHubRepos();
  }
}

export function closeCloneDialog() {
  cloneDialogOpen.set(false);
  cloneError.set(null);
  repoSearchQuery.set('');
}

export async function loadGitHubRepos() {
  githubReposLoading.set(true);
  try {
    const repos = await githubApi.listRepos();
    githubRepos.set(repos);
  } catch (e) {
    cloneError.set(String(e));
  } finally {
    githubReposLoading.set(false);
  }
}

export async function cloneRepository(url: string, dest: string) {
  cloning.set(true);
  cloneError.set(null);
  try {
    const result = await repoApi.cloneRepo(url, dest);
    addOutput('clone', result.output.stdout, result.output.stderr, true);
    closeCloneDialog();
    await openRepo(result.path);
  } catch (e) {
    cloneError.set(String(e));
    addOutput('clone', '', String(e), false);
  } finally {
    cloning.set(false);
  }
}
