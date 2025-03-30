# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.15.1](https://github.com/drewzemke/tongo/compare/v0.15.0...v0.15.1) - 2025-03-30

### Fixed

- *(cd)* correct path to built binary

## [0.15.0](https://github.com/drewzemke/tongo/compare/v0.14.1...v0.15.0) - 2025-03-30

### Added

- *(connections)* add a message to connections screen when there aren't any connections
- *(config)* allow configuring page size
- *(error)* add crate error type, clean up error printing a bit

### Fixed

- *(demo)* rerun with fresh local db
- *(status-bar)* automatically expand status bar to fit error content
- *(tabs)* fix incorrect cloning of `focus` field when duplicating tabs
- *(keys)* properly setup key map with default for "goto tab"
- *(deps)* unpin `deranged` since the problematic version was yanked

### Other

- *(cd)* add workflow to build binaries for releases
- *(themes)* add theme files to repo
- *(readme)* add quick start and some notes on contributing
- *(readme)* add a few extra "feature" entries
- *(readme)* add demo clip
- *(config)* add explanatory comments
- *(config)* [**breaking**] change underscores to hyphens in the key map
- *(key-map)* remove some

## [0.14.1](https://github.com/drewzemke/tongo/compare/v0.14.0...v0.14.1) - 2025-03-24

### Added

- *(themes)* enable theming for pretty much the entire rest of the app
- *(themes)* add all the color keys necessary to theme lists
- *(themes)* add all the color keys necessary to stylize inputs
- *(themes)* allow theme to include a "palette" section where color keys may be defined
- *(themes)* expand valid color strings in theme (to anything that's recognized by `ratatui::style::Color`)
- *(themes)* initial impl, using `theme.toml` to customize doc component colors
- *(themes)* create basic `ColorMap` that gets parsed from the config
- *(fuzzy-search)* allow doc navigation in search review mode

### Fixed

- *(deps)* pin version of `deranged` to 0.4.0
- *(keys)* don't always quit on ctrl-c
- *(cli)* restrict using `--last` in conjunction with `--connection` or `--url`
- *(connection)* correctly set selected connection when loading with `-c`
- *(navigation)* "back" command focuses coll list from documents component (when in normal mode)
- *(app)* process commands on hydrate so they're available for the first draw
- *(config)* allow empty config file
- *(documents)* properly setup component when cloning
- *(documents)* restore refresh command

### Other

- *(color-map)* clean things up, also resolve that giant function that parsed the theme configuration
- *(themes)* separate `documents` section of theme into `data` and `theme`
- *(themes)* add all of the color keys necessary for the documents component
- *(key-map)* move to be submodule of `config`
- *(key-map)* type safety for `KeyMap::default()`
- *(config)* replace the shared `KeyMap` in components with `Config` that houses a `KeyMap`
- *(config)* rename `Config` -> `RawConfig`, improve some error messaging
- *(fuzzy-search)* reset search when done searching

## [0.14.0](https://github.com/drewzemke/tongo/compare/v0.13.0...v0.14.0) - 2025-03-19

### Added

- *(fuzzy-search)* cycle through results in search review mode
- *(fuzzy-search)* search using document keys and key paths
- *(fuzzy-search)* works with nested arrays
- *(fuzzy-search)* fuzzy search selects the top match
- *(key-map)* support and control and alt in keybindings
- *(documents)* clear data when selected connection is invalidated
- *(help)* enable executing the selected command :)
- *(help)* show cursor in help modal
- *(help)* cursor movement logic in help modal
- *(status-bar)* show app name and version in the bottom right corner
- *(status-bar)* only show certain commands in the status bar
- *(help)* give categories to every command
- *(help)* much-improved layout of the help popup
- *(help)* a little more styling on the help menu
- *(help)* add categories to commands, WIP (and I mean it) of rendering commands grouped by category in the modal
- *(help)* super basic impl of a help modal that shows all currently-available commands
- *(help)* initial impl of a help modal
- *(databases)* create and drop databases
- *(collections)* add collections to a db

### Fixed

- *(fuzzy-search)* don't use tokio to inject docs if no runtime is present (eg. in tests)
- *(key-map)* render key mappings with modifiers in app
- *(tabs)* correct tab name when starting app with `--connection`
- *(tabs)* only allow some tab commands if there's more than one tab
- *(documents)* only make doc-specific command available when a document is selected
- *(connections)* constrain size of connection editor
- *(lints)* turn clippy pedantic lints back on, and fix (almost) all of them

### Other

- *(fuzzy-search)* load docs into `nucleo` concurrently
- *(fuzzy-search)* starter impl of fuzzy search in docs
- *(key-map)* convert some functions into trait implementations
- *(key_map)* replace `KeyCode` with a `Key` type that encapsulates it
- *(json-labeler)* remove obsolete test
- *(commands)* touch up help modal text
- *(components)* add extra spaces around block titles
- *(help)* use a vec instead of a hashmap to store cached categorized groups
- *(help)* remove state from help modal (for now)
- *(commands)* move string->command function into `key_map` module
- *(commands)* introduce `CommandManager` to manage command statee
- *(model)* use simpler `Collection` and `Database` structs instead of `CollectionSpecification` and `DatabaseSpecification` from mongodb
- *(modules)* move `connection` into new `model` module
- *(components)* convert `Tab.focus` into a `Cell`
- *(components)* remove the `ComponentCommand` enum
- *(components)* add `handle_raw_event` fn to `Component` trait
- *(messages)* change one last event into a message, also add doc comments for all the messages
- *(events)* more doc comments and some renaming of events
- *(components)* add default impls for all `Component` trait functions
- *(events)* doc comments for some of the events
- *(messages)* change events that are target at `Tab` and `ConnectionScreen` into messages
- *(messages)* make message creation/reading more idiomatic
- *(messages)* change events that are targeted at `Client` into messages
- *(messages)* change events that are target at `App` into messages
- *(message)* introduce `Message` and `Signal` to help organize component comms system
- *(system)* add some comments in planning for event/message refactor
- *(collections)* most of the impl for creating new collections; just missing the mongo part
- *(collections)* small test for dropping collection

## [0.13.0](https://github.com/drewzemke/tongo/compare/v0.12.3...v0.13.0) - 2025-03-03

### Added

- *(collections)* ability to drop collections
- *(tabs)* ability to duplicate the current tab
- *(tabs)* tab names update to reflect current db and collection
- *(tabs)* tab names show connection names
- *(tabs)* close tabs
- *(tabs)* jump to tab by number
- *(tabs)* persist `TabBar`'s state
- *(tabs)* scroll tab bar to keep current tab visible
- *(tabs)* show tabs at top of window
- *(tabs)* add start of a tab bar -- not visible but manages tab state
- *(tabs)* create a new tabs and switch between them
- *(tabs)* convert `App.tab` into `App.tabs` (a list of tabs)

### Fixed

- *(app)* render content in the correct container when no tabs are visible
- *(confirm-modal)* constrain width independent of screen size
- *(documents)* setup default state correctly when cloning `Documents` component
- *(tabs)* resolve some scrolling issues in `TabBar`

### Other

- *(commands)* [**breaking**] merge some commands
- *(confirm-modal)* introduce new enum to distinguish between confirmations
- *(project)* add license
- *(components)* add `Clone` implementations for most components
- *(status-bar)* move `StatusBar` to be a child component of `App` rather than `Tab`
- *(tabs)* use `get` to access tabs
- *(connections)* introduce `ConnectionsManager` to managed shared connection state
- *(tabs)* split a `Tab` component out of app that holds all of the main components

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
