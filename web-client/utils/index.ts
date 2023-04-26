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
