/* eslint-disable import/prefer-default-export */
import semverCompare from 'semver-compare';
import { Category, Version } from '../interfaces/Package';

export const compareVersions = (a: Version, b: Version) =>
  semverCompare(a.version, b.version);

export const sortCategories = (a: Category, b: Category) => {
  if (a.name < b.name) return -1;
  if (a.name > b.name) return 1;
  return 0;
};
