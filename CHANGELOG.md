# Changelog

All notable changes to hb-subset-rs will be documented in this file.

## [0.2.1] - 2023-11-06

### Bug Fixes

- Remove unformated C documentation from sys

### Features

- Make Set API general
- Overall improvements to Set API
- Make SetIter DoubleEndedIterator
- Sets can now be cloned

### Miscellaneous Tasks

- Add doc aliases to method calls
- Simplify SetIter
- Add aliases for hb_set_next and hb_set_previous
- Add CHANGELOG.md

## [0.2.0] - 2023-11-04

### Miscellaneous Tasks

- Add ci script

- Merge pull request #2 from henkkuli/ci

Add CI basic scripts
- Extract mid-level bindings to their own module

This makes room for introducing high-level bindings later.


This changelog was generated automatically.
To  update it, run `git cliff`.
