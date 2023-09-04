/* eslint-disable import/prefer-default-export */
import semverCompare from 'semver-compare';
import { Version } from '../interfaces/Package';

export const compareVersions = (a: Version, b: Version) =>
  semverCompare(a.version, b.version);
