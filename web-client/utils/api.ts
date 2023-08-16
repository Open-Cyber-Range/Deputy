import { Fetcher } from 'swr';
import TOML from '@iarna/toml';
import { getSession } from 'next-auth/react';
import { Package, PackageWithVersions, Version } from '../interfaces/Package';
import { compareVersions } from './sort';
import { Project } from '../interfaces/Project';
import { ModifiedSession, PostToken, Token } from '../interfaces/Token';

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

export const createToken = async (postToken: PostToken) => {
  const session = (await getSession()) as ModifiedSession;
  if (session && session.idToken) {
    const response = await fetch('/api/v1/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${session.idToken}`,
      },
      body: JSON.stringify(postToken),
    });

    if (response.ok) {
      const token = (await response.json()) as unknown as Token;
      return token;
    }

    throw new Error('Failed to create token');
  }

  throw new Error('No session found');
};
