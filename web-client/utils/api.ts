import { Fetcher } from 'swr';
// eslint-disable-next-line import/no-extraneous-dependencies
import TOML from '@iarna/toml';
import { Package, PackageWithVersions, Version } from '../interfaces/Package';
import { compareVersions } from './sort';
import { Project } from '../interfaces/Project';

export const packagesWithVersionsFetcher: Fetcher<
  PackageWithVersions[],
  string
> = async (...url) => {
  const packages: Package[] = await fetch(...url).then(async (res) =>
    res.json()
  );

  return Promise.all(
    packages.map(async (pkg) => {
      const response = await fetch(`${url}/${pkg.name}/`);

      const packageWithVersions: PackageWithVersions = {
        ...pkg,
        versions: (await response.json()).sort(compareVersions),
      };
      return packageWithVersions;
    })
  );
};

export const packageVersionsFethcer: Fetcher<Version[], string> = async (
  ...url
) => {
  const response = await fetch(...url);
  return response.json();
};

export const packageVersionFethcer: Fetcher<Version, string> = async (
  ...url
) => {
  const response = await fetch(...url);
  return response.json();
};

export const packageTOMLFetcher: Fetcher<Project, string> = async (...url) => {
  const response = await fetch(...url);
  return TOML.parse(await response.text()) as unknown as Project;
};
