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
  readme_path: string;
  readme_html: string;
  checksum: string;
  created_at: string;
  updated_at: string;
};

export type PackageWithVersions = Package & {
  versions: Version[];
};
