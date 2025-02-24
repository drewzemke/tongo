# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.3](https://github.com/drewzemke/tongo/compare/v0.12.2...v0.12.3) - 2025-02-24

### Fixed

- *(filter)* actually bundle the extended json syntax file with the program instead of expecting it be found locally

## [0.12.2](https://github.com/drewzemke/tongo/compare/v0.12.1...v0.12.2) - 2025-02-24

### Added

- *(filter)* show error when filter is not valid
- *(filter)* show a symbol indicating if the filter is syntatically valid
- *(persistence)* store/restore the document filter
- *(persistence)* persist `Client` state
- *(persistence)* store/restore list and doc component items rather than querying them after hydrating
- *(filter)* expanded syntax for filter input
- *(connections)* ability to edit connections
- *(testing)* add `MockStorage` to stop tests from writing to the filesystem :D

### Other

- *(readme)* lil' updates
- *(persistence)* remove feature flag since the thing is working now :)
- *(persistence)* `PersistedComponent::hydrate` no longer generates events
- *(persistence)* remove `pending_selection` state from lists and doc view
- *(modules)* rename module `sessions` -> `persistence`
- *(modules)* move top-level module defns to `lib.rs`
- *(packages)* bump more packages (incompatible changes*)
- *(packages)* bump `mongodb` to 3.2.1
- *(packages)* bump `ratatui` to 0.29.0
- *(packages)* bump packages (compatible changes)
- *(connections)* connections have ids
- *(storage)* add `Storage` trait to decouple components from filesystem
- *(components)* add some missing `Debug` and `Clone` derive macros
- *(project)* rename module `files` -> `file_manager`

## [0.12.1](https://github.com/drewzemke/tongo/compare/v0.12.0...v0.12.1) - 2025-02-19

### Added

- *(nav)* automatically select the first item of a list when focusing it for the first time

### Other

- *(cleanup)* resolved a couple TODOs
- *(files)* dependency inject FileManager and make it's functions more concrete
- *(features)* enable all features in helix (and in CI)
- *(changelog)* manually update

## [0.12.0](https://github.com/drewzemke/tongo/compare/v0.10.1...v0.12.0) - 2025-02-18

### Added

- *(config)* create a default config file if one is not present
- *(key-map)* [**breaking**] override default keys with a config file
- *(events)* add event for whole app losing/gaining focus
- *(sessions)* store/load selected doc
- *(sessions)* store/load document page
- *(sessions)* store/load selected db/coll in lists
- *(sessions)* store/load previously selected db and collections (still sorta buggy)
- *(sessions)* store/load selected connection
- *(sessions)* initial impl of storing/loading previous sessions
- *(confirm)* better confirmation messages
- *(confirm)* show confirm modal for deleting documents
- *(confirm)* initial impl of a confirmation modal
- *(docs)* commands to jump to first or last page of docs

### Fixed

- *(ci)* correct cargo registry token
- *(key-map)* allow one key to be mapped to multiple commands
- *(connections)* clear input values after creating new connections
- *(rendering)* redraw screen when terminal window size changes
- *(sessions)* run count query when loading session
- *(nav)* resolve panic when changing focus in primary screen
- *(ci)* add correct(?) token to checkout in release-plz workflow
- *(collections)* enable selecting a collection when there's only one in the list
- *(perf)* prevent second query when selecting a collection
- *(ci)* allow release-plz workflow to trigger other workflows
- *(ci)* correct command for tests

### Other

- *(sessions)* put session functionality behind a feature flag
- *(key-map)* add a few more options for binding keys
- *(key-map)* create `KeyMap` to maintain key configuration
- *(app)* shorten focus enum names
- *(dev-ex)* add docker-compose and seed script
- *(clippy)* address new lints
- *(testing)* add test harness to make component testing easier :)
- *(commands)* don't pass crossterm event by reference
- *(input)* `InputComponent` encapsulates its state better
- *(lists)* dimmer selected highlight color for unfocused widgets
- *(justfile)* fix `logs` script
- *(connections)* render new connection inputs in an overlay
- *(sessions)* client doesn't save any state
- *(events)* client doesn't assume page changes when processing certain events
- *(justfile)* add stuff to accommodate logging
- *(logs)* add basic logging setup with `tracing`
- *(sessions)* app hydration process produces/uses events
- *(data)* `Client` and `Documents` can hold on to their own copies of the page value without causing extra queries
- *(client)* execute (deduped) queries after all events have been processed (per command/event loop)
- *(sessions)* slightly-more-graceful error handling when loading stored session
- *(sessions)* don't save app focus as any of the input components
- *(client)* rename functions and use `Option`/`Result` to (hopefully) improve readability
- *(lints)* turn `allow` attributes into `expect`s
- *(data)* use `Cell` instead of `RefCell` for shared cursor position
- *(ci)* only run CI checks on pull-request changes (ie. stuff from `release-plz`)
- *(ci/cd)* add initial CI workflow to check stuff on push

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
