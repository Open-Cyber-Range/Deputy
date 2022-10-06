interface Package {
  name: string;
  description: string;
  version: string;
  authors: string[];
}

interface Content {
  type: string;
}

interface Account {
  name: string;
  password: string;
}

interface VirtualMachine {
  accounts: Account[];
  default_account: string;
  operating_system: string;
  architecture?: any;
  type: string;
  file_path: string;
  readme_path?: any;
}

enum FeatureType {
  SERVICE = 'SERVICE',
  CONFIGURATION = 'CONFIGURATION',
  ARTIFACT = 'ARTIFACT',
}

interface Feature {
  feature_type: FeatureType;
}

export interface PackageList {
  package: Package;
  content: Content;
  virtual_machine: VirtualMachine;
  feature: Feature;
}