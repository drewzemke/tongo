# Signal System Refactor Plan

## Overview

This document outlines the plan to refactor tongo's signal system to use the improved `SignalQueue` pattern from the todoist/tui project. This refactor will improve the API design and reduce code complexity while maintaining the same functional behavior.

## Current System Analysis

### Current State (tongo)
- Components return `Vec<Signal>` from handler methods
- App manually collects signals and manages a `VecDeque<Signal>`
- Signal processing loops through the deque with `pop_front()`
- New signals from handlers are manually added back to the deque

### Target State (todoist pattern)
- Dedicated `SignalQueue` struct wraps the queue management
- Components receive `&mut SignalQueue` parameter directly
- Components push signals directly using `queue.push(signal)`
- Cleaner API with better encapsulation

## Benefits of Refactor

1. **More Intuitive API**: Components directly push signals instead of collecting in vectors
2. **Better Encapsulation**: SignalQueue handles internal queue management  
3. **Reduced Allocations**: No Vec<Signal> creation for each handler call
4. **Consistency**: Matches pattern used in newer todoist project
5. **Cleaner Code**: Simplified signal emission during processing

## Implementation Plan

### Phase 1: Core Infrastructure

#### Step 1: Add SignalQueue to system.rs
- Port the `SignalQueue` implementation from `../todoist/tui/src/system/signal.rs`
- Add it to `src/system.rs` alongside the existing `Signal` enum
- Ensure it has the same API: `push()`, `pop()`, and `Default` implementation

#### Step 2: Update Component Trait
**File**: `src/components.rs`
- Change method signatures from returning `Vec<Signal>` to accepting `&mut SignalQueue`
- Update trait methods:
  - `handle_command(&mut self, command: &Command, queue: &mut SignalQueue)`
  - `handle_raw_event(&mut self, event: &CrosstermEvent, queue: &mut SignalQueue)`
  - `handle_event(&mut self, event: &Event, queue: &mut SignalQueue)`
  - `handle_message(&mut self, message: &Message, queue: &mut SignalQueue)`

### Phase 2: Core App Changes

#### Step 3: Refactor App::process_signals
**File**: `src/app.rs`
- Replace manual `VecDeque` management with `SignalQueue`
- Update method signature: `process_signals(&mut self, queue: &mut SignalQueue)`
- Change signal processing loop to use `queue.pop()` instead of `signals_deque.pop_front()`
- Remove manual signal collection and deque management

#### Step 4: Update App signal creation
**File**: `src/app.rs`
- Update the main event loop to use `SignalQueue`
- Change from collecting `Vec<Signal>` to pushing directly to queue
- Update initial signal creation and tick handling

### Phase 3: Component Updates

#### Step 5: Update All Component Implementations
The following components need their handler methods updated:

**Core Components:**
- `src/components/tab.rs` - Tab component signal handling
- `src/components/primary_screen.rs` - Primary screen navigation
- `src/components/connection_screen.rs` - Connection management
- `src/components/documents.rs` - Document operations
- `src/components/status_bar.rs` - Status updates
- `src/components/tab_bar.rs` - Tab management
- `src/components/help_modal.rs` - Help modal interactions
- `src/components/confirm_modal.rs` - Confirmation dialogs

**Nested Components:**
- `src/components/input/*.rs` - Input handling components
- `src/components/list.rs` - List navigation

**Change Pattern:**
```rust
// OLD:
fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
    vec![Signal::Event(Event::SomeEvent)]
}

// NEW:
fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
    queue.push(Event::SomeEvent);
}
```

#### Step 6: Update Client Integration
**File**: `src/client.rs`
- If Client implements Component trait, update its signal handling
- Ensure async operation results properly push to the queue

### Phase 4: Testing and Validation

#### Step 7: Compile and Test
- Run `cargo check` to verify compilation
- Run `cargo test` to ensure tests pass
- Manual testing of key workflows:
  - Connection management
  - Database/collection navigation
  - Document CRUD operations
  - Tab management
  - Modal interactions

#### Step 8: Integration Testing
- Test complex signal chains (e.g., user action → multiple events → UI updates)
- Verify async operations still work correctly
- Test error scenarios and modal workflows

## Migration Notes

### Breaking Changes
- All component implementations must be updated simultaneously
- The Component trait API changes are breaking changes
- Any custom components outside the main codebase would need updates

### Backward Compatibility
- The `Signal`, `Event`, and `Message` types remain unchanged
- The functional behavior of signal processing is preserved
- Only the API for signal emission changes

## Potential Issues

1. **Missed Component**: If any component isn't updated, compilation will fail
2. **Signal Chains**: Complex signal chains need careful testing
3. **Async Boundaries**: Ensure async operations properly integrate with new system

## Progress Update

### ✅ Completed
- [x] **Phase 1: Core Infrastructure** - SignalQueue added to system, Component trait updated
- [x] **Phase 2: App Changes** - App::process_signals refactored, App Component implementation updated

### 🔄 In Progress  
- **Phase 3: Component Updates** - The following components need their handler methods updated:

**Client:**
- `src/client.rs` - handle_event, handle_message

**Core Components:**
- `src/components/confirm_modal.rs` - handle_command
- `src/components/connection_screen.rs` - handle_command, handle_raw_event, handle_event, handle_message  
- `src/components/documents.rs` - handle_command, handle_event, handle_message
- `src/components/help_modal.rs` - handle_command, handle_event
- `src/components/list.rs` - handle_command, handle_event  
- `src/components/primary_screen.rs` - handle_command, handle_event, handle_message
- `src/components/status_bar.rs` - handle_event, handle_message
- `src/components/tab.rs` - handle_command, handle_raw_event, handle_event, handle_message
- `src/components/tab_bar.rs` - handle_command, handle_event

**Input Components:**
- `src/components/input/filter_input.rs` - handle_command, handle_raw_event, handle_event
- `src/components/input/input_modal.rs` - handle_command, handle_raw_event, handle_event
- `src/components/input/text_input.rs` - handle_command, handle_raw_event

## Success Criteria

- [x] All code compiles without warnings  
- [ ] All existing tests pass
- [ ] Manual testing confirms same functionality
- [ ] No performance regressions
- [ ] Code is cleaner and more maintainable

## Files to Modify

### Core Files
- `src/system.rs` - Add SignalQueue
- `src/components.rs` - Update Component trait
- `src/app.rs` - Update signal processing

### Component Files
- `src/components/tab.rs`
- `src/components/primary_screen.rs`
- `src/components/connection_screen.rs`
- `src/components/documents.rs`
- `src/components/status_bar.rs`
- `src/components/tab_bar.rs`
- `src/components/help_modal.rs`
- `src/components/confirm_modal.rs`
- `src/components/list.rs`
- `src/components/input/*.rs` (all input components)
- `src/client.rs` (if applicable)

### Estimated Effort
- **Planning**: ✅ Complete
- **Core Infrastructure**: ~2 hours
- **Component Updates**: ~4 hours  
- **Testing & Validation**: ~2 hours
- **Total**: ~8 hours

This refactor improves code quality while maintaining all existing functionality.