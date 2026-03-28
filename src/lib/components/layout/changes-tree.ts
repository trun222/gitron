import type { FileStatusType } from '$lib/api/types';
import type { FileSection } from '$lib/stores/repo';

export interface TreeNode {
  name: string;
  path: string;
  type: 'file' | 'dir';
  children?: TreeNode[];
  fileStatus?: FileStatusType;
  section?: FileSection;
  fileCount?: number;
}

export interface FlatEntry {
  name: string;
  path: string;
  type: 'file' | 'dir';
  depth: number;
  fileStatus?: FileStatusType;
  section?: FileSection;
  expanded?: boolean;
  fileCount?: number;
}

interface FileInput {
  path: string;
  status?: FileStatusType;
  section: FileSection;
}

export function buildTree(files: FileInput[]): TreeNode[] {
  const root: TreeNode[] = [];

  for (const file of files) {
    const parts = file.path.split('/');
    let current = root;

    for (let i = 0; i < parts.length; i++) {
      const name = parts[i];
      const isFile = i === parts.length - 1;
      const partPath = parts.slice(0, i + 1).join('/');

      if (isFile) {
        current.push({
          name,
          path: file.path,
          type: 'file',
          fileStatus: file.status,
          section: file.section,
        });
      } else {
        let dir = current.find((n) => n.type === 'dir' && n.name === name);
        if (!dir) {
          dir = { name, path: partPath, type: 'dir', children: [] };
          current.push(dir);
        }
        current = dir.children!;
      }
    }
  }

  sortTree(root);
  computeFileCounts(root);
  return root;
}

function computeFileCounts(nodes: TreeNode[]): number {
  let total = 0;
  for (const node of nodes) {
    if (node.type === 'file') {
      total++;
    } else if (node.children) {
      node.fileCount = computeFileCounts(node.children);
      total += node.fileCount;
    }
  }
  return total;
}

function sortTree(nodes: TreeNode[]): void {
  nodes.sort((a, b) => {
    if (a.type !== b.type) return a.type === 'dir' ? -1 : 1;
    return a.name.localeCompare(b.name);
  });
  for (const node of nodes) {
    if (node.children) sortTree(node.children);
  }
}

export function flattenTree(nodes: TreeNode[], expandedDirs: Set<string>, depth = 0): FlatEntry[] {
  const result: FlatEntry[] = [];

  for (const node of nodes) {
    if (node.type === 'dir') {
      const expanded = expandedDirs.has(node.path);
      result.push({
        name: node.name,
        path: node.path,
        type: 'dir',
        depth,
        expanded,
        fileCount: node.fileCount,
      });
      if (expanded && node.children) {
        result.push(...flattenTree(node.children, expandedDirs, depth + 1));
      }
    } else {
      result.push({
        name: node.name,
        path: node.path,
        type: 'file',
        depth,
        fileStatus: node.fileStatus,
        section: node.section,
      });
    }
  }

  return result;
}

/** Collect all directory paths from a tree for default-expand-all behavior. */
export function collectDirPaths(nodes: TreeNode[]): string[] {
  const paths: string[] = [];
  for (const node of nodes) {
    if (node.type === 'dir') {
      paths.push(node.path);
      if (node.children) paths.push(...collectDirPaths(node.children));
    }
  }
  return paths;
}
