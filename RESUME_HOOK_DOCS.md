# Resume Hook Feature

This document describes the new resume hook functionality added to mlua.

## Overview

The resume hook feature allows you to set up hooks that trigger when a coroutine/thread resumes execution. This is useful for:

- Debugging coroutine execution flow
- Performance monitoring of async operations
- Logging coroutine lifecycle events
- Implementing custom scheduling or profiling tools

## Usage

### Basic Resume Hook

```rust
use mlua::{DebugEvent, HookTriggers, Lua, Result, VmState};

let lua = Lua::new();

// Set a global hook that triggers when coroutines resume
lua.set_global_hook(HookTriggers::ON_RESUME, |_lua, debug| {
    println!("Coroutine resumed! Event: {:?}", debug.event());
    Ok(VmState::Continue)
})?;

// Create and use a coroutine
let thread = lua.create_thread(
    lua.load("coroutine.yield(42)").into_function()?
)?;

// This will trigger the resume hook
let result: i32 = thread.resume(())?;
```

### Combined Hooks

You can combine resume hooks with other hook types:

```rust
// Hook that triggers on both function calls and resumes
lua.set_global_hook(
    HookTriggers::ON_CALLS | HookTriggers::ON_RESUME,
    |_lua, debug| {
        match debug.event() {
            DebugEvent::Call => println!("Function called"),
            DebugEvent::Resume => println!("Coroutine resumed"),
            _ => {}
        }
        Ok(VmState::Continue)
    }
)?;
```

### Thread-Specific Hooks

Resume hooks also work with thread-specific hooks:

```rust
// Set a resume hook for a specific thread
thread.set_hook(HookTriggers::ON_RESUME, |_lua, debug| {
    println!("This specific thread resumed");
    Ok(VmState::Continue)
})?;
```

## Debug Information for Resume Events

When a resume hook is triggered, the `Debug` object provides limited information since resume is not a native Lua debug event:

- `debug.event()` returns `DebugEvent::Resume`
- `debug.source()` returns synthetic source information with `what = "resume"`
- `debug.names()`, `debug.current_line()`, `debug.stack()` return default/empty values
- `debug.function()` will panic if called (resume events don't have associated functions)

## Implementation Notes

- Resume hooks are implemented at the mlua level, not at the Lua C API level
- They only trigger when using mlua's `Thread::resume()` method
- Resume hooks cannot yield (unlike some other hook types)
- This feature is only available when not using the "luau" feature

## Constants and Methods

### New HookTriggers Field

```rust
pub struct HookTriggers {
    // ... existing fields ...
    
    /// When a coroutine/thread resumes execution.
    pub on_resume: bool,
}
```

### New Constant

```rust
impl HookTriggers {
    /// An instance of `HookTriggers` with `on_resume` trigger set.
    pub const ON_RESUME: Self = HookTriggers::new().on_resume();
}
```

### New Method

```rust
impl HookTriggers {
    /// Returns an instance of `HookTriggers` with `on_resume` trigger set.
    pub const fn on_resume(mut self) -> Self;
}
```

### New Debug Event

```rust
pub enum DebugEvent {
    // ... existing variants ...
    
    /// Custom event when a coroutine/thread resumes execution.
    /// This is not a native Lua debug event.
    Resume,
}
```

## Examples

See the examples directory for complete working examples:
- `resume_hook_example.rs` - Basic resume hook usage
- `combined_hooks_example.rs` - Combining resume hooks with other hook types
