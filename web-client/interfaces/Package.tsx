export type Package = {
  id: string;
  name: string;
  description: string;
  readme_html: string;
  created_at: string;
};

export type Version = {
  id: string;
  version: string;
  license: string;
  is_yanked: boolean;
  readme_path: string;
  readme_html: string;
  package_size: number;
  checksum: string;
  created_at: string;
  updated_at: string;
};

export type PackageWithVersions = {
  id: string;
  name: string;
  description: string;
  readme_html: string;
  created_at: string;
  versions: Version[];
};

export type PackagesWithPages = {
  packages: Package[];
  total_pages: number;
};

export type PackagesWithVersionsAndPages = {
  packages: PackageWithVersions[];
  total_pages: number;
};
