<script lang="ts">
  import '../app.css';
  import type { Snippet } from 'svelte';
  import DirectoryBrowser from '$lib/components/ui/dialog/DirectoryBrowser.svelte';

  let { children }: { children: Snippet } = $props();
</script>

{@render children()}
<DirectoryBrowser />
