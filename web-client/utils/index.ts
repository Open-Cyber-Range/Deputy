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
