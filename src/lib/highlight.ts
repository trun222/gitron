import { createHighlighter, type Highlighter, type ThemedToken, type BundledLanguage } from 'shiki';

const THEME = 'catppuccin-mocha';

const PRELOADED_LANGS = [
  'javascript',
  'typescript',
  'jsx',
  'tsx',
  'rust',
  'json',
  'toml',
  'yaml',
  'html',
  'css',
  'svelte',
  'markdown',
  'bash',
  'python',
] as const;

const EXT_TO_LANG: Record<string, string> = {
  js: 'javascript',
  mjs: 'javascript',
  cjs: 'javascript',
  jsx: 'jsx',
  ts: 'typescript',
  mts: 'typescript',
  cts: 'typescript',
  tsx: 'tsx',
  rs: 'rust',
  py: 'python',
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  toml: 'toml',
  html: 'html',
  htm: 'html',
  css: 'css',
  svelte: 'svelte',
  md: 'markdown',
  sh: 'bash',
  bash: 'bash',
  zsh: 'bash',
};

let highlighterPromise: Promise<Highlighter> | null = null;

export function getHighlighter(): Promise<Highlighter> {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: [THEME],
      langs: [...PRELOADED_LANGS],
    });
  }
  return highlighterPromise;
}

export function detectLanguage(filePath: string): string {
  const ext = filePath.split('.').pop()?.toLowerCase() ?? '';
  return EXT_TO_LANG[ext] ?? 'text';
}

export function tokenizeLine(
  highlighter: Highlighter | null,
  content: string,
  lang: string,
): ThemedToken[] {
  const trimmed = content.replace(/\n$/, '');
  if (!highlighter || lang === 'text' || trimmed === '') {
    return [{ content: trimmed, color: undefined, offset: 0 }];
  }
  try {
    const result = highlighter.codeToTokens(trimmed, { lang: lang as BundledLanguage, theme: THEME });
    return result.tokens[0] ?? [{ content: trimmed, color: undefined, offset: 0 }];
  } catch {
    return [{ content: trimmed, color: undefined, offset: 0 }];
  }
}
