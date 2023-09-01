import { PackageWithVersions, Version } from '../interfaces/Package';

export const getLatestVersion = (pkg: PackageWithVersions): Version | null => {
  if (pkg.versions.length > 0) {
    return pkg.versions[pkg.versions.length - 1];
  }

  return null;
};

export const displayLocalTime = (iso8601Date: string): string => {
  const date = new Date(iso8601Date);
  return date.toLocaleString();
};

export function formatBytes(bytes: number, decimals = 2): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${(bytes / k ** i).toFixed(dm)} ${sizes[i]}`;
}

export function extractAndRemoveTypeAndCategory(inputString: string): {
  type: string | null;
  categories: string | null;
  remainingString: string;
} {
  const typeRegex = /type:(\w+)/;
  const categoryRegex = /categories:([^ ]+)/;
  let remainingString = inputString;
  let type: string | null = null;
  let categories: string | null = null;

  const matchType = inputString.match(typeRegex);
  if (matchType) {
    [, type] = matchType;
    remainingString = remainingString.replace(typeRegex, '').trim();
  }

  const matchCategory = inputString.match(categoryRegex);
  if (matchCategory) {
    [, categories] = matchCategory;
    remainingString = remainingString.replace(categoryRegex, '').trim();
  }
  return { type, categories, remainingString };
}
