import md5 from 'md5';

const cache = new Map<string, string>();

export function gravatarUrl(email: string, size = 20): string {
  const key = `${email}:${size}`;
  const cached = cache.get(key);
  if (cached) return cached;

  const hash = md5(email.trim().toLowerCase());
  const url = `https://www.gravatar.com/avatar/${hash}?s=${size}&d=404`;
  cache.set(key, url);
  return url;
}
