import semverCompare from 'semver-compare';
import { PackageWithVersions, Version } from '../interfaces/Package';

export const compareVersions = (a: Version, b: Version) =>
  semverCompare(a.version, b.version);

export const sortPackagesByName = (packages: PackageWithVersions[]) => {
  packages.sort((a, b) => {
    if (a.name < b.name) {
      return -1;
    }
    if (a.name > b.name) {
      return 1;
    }
    return 0;
  });
};
