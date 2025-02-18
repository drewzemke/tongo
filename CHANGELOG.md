# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.12.0](https://github.com/drewzemke/tongo/compare/v0.11.0...v0.12.0) - 2025-02-18

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
- feat(confirm): initial impl of a confirmation modal
- *(docs)* commands to jump to first or last page of docs
- *(edit-docs)* better error handling
- *(status-bar)* temporarily display error and other info messages in status bar
- *(yank)* initial impl of yanking docs (or subdocs) to clipboard
- *(performance)* only rerender when a nontrivial event has occurred
- *(docs)* duplicate documents
- *(docs)* insert new documents
- *(query)* filter defaults to "{}", can be reset
- *(query)* syntax highlighting in filter editor
- *(documents)* edit documents in json files
- *(documents)* edit documents in terminal editor
- *(documents)* delete a document
- *(lists)* navigate lists cyclically
- *(session)* mask passwords in connection list
- *(key-hints)* add key hints
- *(docs)* more keys available for navigation in doc view
- *(session)* delete an existing connection
- *(session)* connect to a previously-saved connection from the command line
- *(session)* store/load connections from a file :)
- *(session)* give a name to new connections (still not persisted ._.)
- *(session)* choose from a (hardcoded!) list of connections
- *(session)* connection string command line arg is now optional
- *(session)* initial super-rough impl of setting connection string
- *(error)* show errors in bottom line of app (initial impl, needs more work)
- *(filter)* initial impl of parsing filters from input
- *(filter)* add input bar (doesn't do anything yet)
- *(db)* TUI panes to select db and collection
- *(pagination)* page count with numbers of displayed docs
- *(refresh)* add refresh functionality
- *(pagination)* initial impl of pagination
- top-level document are initially expanded
- *(cli)* add command line args for connection, db, and collection
- package setup and POC

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
- fix(input): bugs with filter input
- *(documents)* restore refresh functionality
- *(list)* prevent panic when navigating empty list
- *(connections)* write updated connections to file on creation
- *(docs)* correctly restore screen when edit-in-external fails
- *(docs)* reset page when executing certain queries
- *(documents)* keep doc tree selection when performing certain content updates
- *(tui)* restore terminal properly when a doc is edited in external editor
- *(lists)* correctly update state when wrapping from top to bottom
- *(pagination)* reset page when selecting a new collection
- *(paging)* don't allow advancing to an empty page
- *(ids)* allow ids of any format

### Other

- manually bump version to 0.11.0
- *(sessions)* put session functionality behind a feature flag
- *(key-map)* add a few more options for binding keys
- *(key-map)* create `KeyMap` to maintain key configuration
- *(app)* shorten focus enum names
- *(dev-ex)* add docker-compose and seed script
- *(clippy)* address new lints
- gitignore
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
- update justfile
- *(client)* rename functions and use `Option`/`Result` to (hopefully) improve readability
- *(lints)* turn `allow` attributes into `expect`s
- *(data)* use `Cell` instead of `RefCell` for shared cursor position
- *(ci)* only run CI checks on pull-request changes (ie. stuff from `release-plz`)
- *(ci/cd)* add initial CI workflow to check stuff on push
- release
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
- refactor(commands): reimplement doc duplication and deletion
- refactor(commands): re-implement doc editing and creating
- refactor(commands): wire up filter input
- refactor(commands): wire up documents view
- refactor(commands): add focus management to list components
- *(commands)* rename list components
- refactor(commands): wire up db and collection lists
- refactor(commands): wire up connection screen inputs
- refactor(commands): hook up commands and events for connection screen
- *(commands)* rename `CommandInfo` -> `CommandGroup`
- *(commands)* add initial impl of command and component traits
- *(key-hints)* add prev/next page hints to main doc view
- v0.10.0
- *(docs)* clean up some repeated code in `main_view.rs`
- v0.9.0
- v0.8.1
- v0.8.0
- *(documents)* newtype for document ids, refactored display functions
- some lints
- *(input)* add event for when input editing starts
- *(readme)* update cli instructions
- v0.7.0
- *(input)* extract input widget functionality into trait
- *(lists)* implement `ListWidget` for collection list
- *(lists)* implement `ListWidget` for collection list
- *(lists)* add `ListWidget` trait, implement for db list
- v0.6.0
- *(session)* default connection string field to current connection when passed from command line
- *(package)* version bump
- *(session)* slightly nicer-looking connection screen
- *(input)* localize cursor position management to input widget
- *(screens)* introduce notion of "screens" to organize state/rendering better
- reorganize file structure
- *(tui)* bump ratatui version, simplify some things
- *(package)* crate name change
- *(readme)* update with nix installation instructions
- *(publish)* version bump
- *(nix)* basic nix config
- *(crate)* rename binary to `tongo`
- *(publish)* crate version bump
- *(widgets)* widgets have their own sub-states
- *(state)* (re)introduce `mode` with a new connotation
- *(state)* rename `mode` to `focus`
- *(state)* remove some unused stuff from `State` struct
- *(state)* state stores db and collection structs rather than just names
- *(errors)* better handling, dealt with all `unwrap`s
- *(publish)* add details to `Cargo.toml`
- *(readme)* initial readme
- *(app)* move `App` functionality into separate module
- initial commit

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
