interface Package {
  name: string;
  description: string;
  version: string;
  authors: string[];
}

interface Content {
  type: string;
}

interface VirtualMachine {
  operating_system: string;
  architecture?: any;
  type: string;
  file_path: string;
  readme_path?: any;
}

export interface PackageList {
  package: Package;
  content: Content;
  virtual_machine: VirtualMachine;
}