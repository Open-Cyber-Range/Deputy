import semverCompare from 'semver-compare';
import { Version } from '../interfaces/Package';

// eslint-disable-next-line import/prefer-default-export
export const compareVersions = (a: Version, b: Version) =>
  semverCompare(a.version, b.version);
