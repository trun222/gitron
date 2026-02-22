import type { Transport } from './transport';

// Maps Tauri command names to HTTP endpoints and parameter transformations.
// Tauri uses snake_case commands, the server uses kebab-case URL paths.
const COMMAND_MAP: Record<string, string> = {
  // Repo
  open_repo: '/api/repo/open',
  close_repo: '/api/repo/close',
  get_status: '/api/repo/status',
  get_repo_info: '/api/repo/info',
  set_watcher_interval: '/api/repo/watcher-interval',
  // Graph
  get_commit_graph: '/api/graph',
  get_commit_detail: '/api/graph/detail',
  // Diff
  get_workdir_diff: '/api/diff/workdir',
  get_file_diff: '/api/diff/file',
  get_staged_file_diff: '/api/diff/staged',
  // Staging
  stage_file: '/api/staging/stage',
  unstage_file: '/api/staging/unstage',
  stage_files: '/api/staging/stage-many',
  stage_all: '/api/staging/stage-all',
  unstage_all: '/api/staging/unstage-all',
  discard_all_changes: '/api/staging/discard-all',
  discard_files: '/api/staging/discard-files',
  add_to_gitignore: '/api/staging/gitignore',
  // Branch
  list_branches: '/api/branch/list',
  create_branch: '/api/branch/create',
  checkout_branch: '/api/branch/checkout',
  delete_branch: '/api/branch/delete',
  reset_to_commit: '/api/branch/reset',
  rebase_onto: '/api/branch/rebase',
  merge_into: '/api/branch/merge',
  // Commit
  create_commit: '/api/commit',
  // Stash
  apply_stash: '/api/stash/apply',
  pop_stash: '/api/stash/pop',
  drop_stash: '/api/stash/drop',
  // Remote
  list_remotes: '/api/remote/list',
  add_remote: '/api/remote/add',
  remove_remote: '/api/remote/remove',
  get_tracking_status: '/api/remote/tracking',
  fetch_remote: '/api/remote/fetch',
  fetch_all_remotes: '/api/remote/fetch-all',
  push_to_remote: '/api/remote/push',
  pull_from_remote: '/api/remote/pull',
  delete_remote_branch: '/api/remote/delete-branch',
  checkout_remote_branch: '/api/remote/checkout',
  // Tag
  create_tag: '/api/tag/create',
  delete_tag: '/api/tag/delete',
  push_tag: '/api/tag/push',
  delete_remote_tag: '/api/tag/delete-remote',
  list_remote_tags: '/api/tag/list-remote',
  // Clone
  clone_repo: '/api/clone',
  // GitHub
  github_check_auth: '/api/github/check-auth',
  github_start_device_flow: '/api/github/start-flow',
  github_poll_device_flow: '/api/github/poll-flow',
  github_logout: '/api/github/logout',
  github_get_user: '/api/github/user',
  github_list_repos: '/api/github/repos',
  // AI
  ai_get_providers: '/api/ai/providers',
  ai_save_key: '/api/ai/save-key',
  ai_delete_key: '/api/ai/delete-key',
  ai_fetch_models: '/api/ai/fetch-models',
  ai_generate_commit_message: '/api/ai/generate',
  ai_get_settings: '/api/ai/settings',
  ai_save_settings: '/api/ai/settings',
  // Filesystem
  fs_list_directory: '/api/fs/list',
};

// Commands that use GET instead of POST
const GET_COMMANDS = new Set(['ai_get_settings']);

export class HttpTransport implements Transport {
  private baseUrl: string;
  private token: string | null;

  constructor(baseUrl?: string, token?: string) {
    if (!baseUrl && typeof window !== 'undefined') {
      const match = window.location.pathname.match(/^\/t\/\d+\/p\/\d+/);
      if (match) baseUrl = match[0];
    }
    this.baseUrl = baseUrl ?? '';
    this.token = token ?? null;
  }

  async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    const endpoint = COMMAND_MAP[command];
    if (!endpoint) {
      throw new Error(`Unknown command: ${command}`);
    }

    const url = `${this.baseUrl}${endpoint}`;
    const isGet = GET_COMMANDS.has(command);

    const headers: Record<string, string> = {};
    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    let response: Response;
    if (isGet) {
      headers['Accept'] = 'application/json';
      response = await fetch(url, { method: 'GET', headers });
    } else {
      headers['Content-Type'] = 'application/json';
      response = await fetch(url, {
        method: 'POST',
        headers,
        body: JSON.stringify(args ?? {}),
      });
    }

    if (!response.ok) {
      const text = await response.text();
      throw new Error(text || `HTTP ${response.status}`);
    }

    return response.json() as Promise<T>;
  }

  async listen<T>(event: string, handler: (payload: T) => void): Promise<() => void> {
    const url = `${this.baseUrl}/api/events`;
    const eventSource = new EventSource(url);

    const listener = (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data) as T;
        handler(data);
      } catch {
        // Ignore parse errors
      }
    };

    // SSE event names match the Tauri event names (e.g. "repo:status-changed")
    eventSource.addEventListener(event, listener);

    return () => {
      eventSource.removeEventListener(event, listener);
      eventSource.close();
    };
  }

  async openUrl(url: string): Promise<void> {
    window.open(url, '_blank');
  }

  async pickDirectory(_title?: string): Promise<string | null> {
    // In web mode, we dispatch a custom event that the DirectoryBrowser component listens for.
    // The component resolves the promise when the user picks a directory.
    return new Promise<string | null>((resolve) => {
      const event = new CustomEvent('gitron:pick-directory', {
        detail: { resolve },
      });
      window.dispatchEvent(event);
    });
  }
}
