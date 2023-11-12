# Changelog

All notable changes to hb-subset-rs will be documented in this file.

## [0.3.0] - 2023-11-12

### Bug Fixes

- [**breaking**] Set equality requires Eq
- [**breaking**] Remove confusing Clone implementation from SubsetInput
- Examples write to existing folder

### Documentation

- Use ? instead of unwrap
- Use raw pointer instead of raw object
- Reword doc for Set.is_empty()
- Fix link to subset in crate doc

### Features

- Allow editing flags and sets of SubsetInput
- Add getters for font face name strings
- Use more specific error types
- Add safe wrapper for language
- Add font preprocessing for faster subsetting
- Add Map
- Add old_to_new_glyph_mapping to SubsetInput
- Add SubsetPlan
- Allow getting nominal glyph mapping from font face

### Miscellaneous Tasks

- [**breaking**] Remove get_ prefix from function names
- [**breaking**] Loosen bounds on Sets
- Run formatting
- Move Flags implementation to own module

### Refactor

- [**breaking**] Move bindings to the main level of the crate

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
