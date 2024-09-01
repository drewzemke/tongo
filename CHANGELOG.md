# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.1](https://github.com/drewzemke/tongo/compare/v0.10.0...v0.10.1) - 2024-09-01

### Added
- *(edit-docs)* better error handling
- *(status-bar)* temporarily display error and other info messages in status bar
- *(yank)* initial impl of yanking docs (or subdocs) to clipboard
- *(performance)* only rerender when a nontrivial event has occurred

### Fixed
- *(input)* bugs with filter input
- *(documents)* restore refresh functionality
- *(list)* prevent panic when navigating empty list
- *(connections)* write updated connections to file on creation
- *(docs)* correctly restore screen when edit-in-external fails

### Other
- *(ci/cd)* add basic setup for `release-plz`
- *(yank)* add function to get the current bson subtree under the cursor
- *(commands)* determine key hints based on command
- *(commands)* move where some commands/events are handled
- *(component)* no longer generic over `ComponentType`
- *(input)* make `InnerInput` common struct that no longer implements `Component`
- *(lists)* replace `ListComponent` trait with `InnerList` struct
- *(package)* reorganize module structure
- *(navigation)* escape to move between components in primary screen
- *(cleanup)* remove obsolete modules
- *(commands)* reimplement doc duplication and deletion
- *(commands)* re-implement doc editing and creating
- *(commands)* wire up filter input
- *(commands)* wire up documents view
- *(commands)* add focus management to list components
- *(commands)* rename list components
- *(commands)* wire up db and collection lists
- *(commands)* wire up connection screen inputs
- *(commands)* hook up commands and events for connection screen
- *(commands)* rename `CommandInfo` -> `CommandGroup`
- *(commands)* add initial impl of command and component traits
- *(key-hints)* add prev/next page hints to main doc view
