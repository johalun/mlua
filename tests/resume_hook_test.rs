#![cfg(not(feature = "luau"))]

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use mlua::{DebugEvent, HookTriggers, Lua, Result, ThreadStatus, VmState};

#[test]
fn test_resume_hook() -> Result<()> {
    let lua = Lua::new();

    let counter = Arc::new(AtomicI32::new(0));
    let hook_counter = counter.clone();
    
    // Set a global hook that triggers on resume
    lua.set_global_hook(HookTriggers::ON_RESUME, move |_lua, debug| {
        // Verify this is a resume event
        assert_eq!(debug.event(), DebugEvent::Resume);
        
        // Verify source information for resume events
        let source = debug.source();
        assert_eq!(source.what, "resume");
        assert!(source.source.is_some());
        assert!(source.short_src.is_some());
        
        hook_counter.fetch_add(1, Ordering::Relaxed);
        Ok(VmState::Continue)
    })?;

    // Create a thread that will yield
    let thread = lua.create_thread(
        lua.load(
            r#"
            local x = 2 + 3
            coroutine.yield(x)
            local y = x * 2
            return y
        "#,
        )
        .into_function()?,
    )?;

    // First resume - this should trigger the hook
    assert_eq!(thread.resume::<i32>(())?, 5);
    assert_eq!(thread.status(), ThreadStatus::Resumable);
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    // Second resume - this should trigger the hook again
    assert_eq!(thread.resume::<i32>(())?, 10);
    assert_eq!(thread.status(), ThreadStatus::Finished);
    assert_eq!(counter.load(Ordering::Relaxed), 2);

    Ok(())
}

#[test]
fn test_resume_hook_combined() -> Result<()> {
    let lua = Lua::new();

    let counter = Arc::new(AtomicI32::new(0));
    let hook_counter = counter.clone();
    
    // Set a hook that triggers on both calls and resume
    lua.set_global_hook(
        HookTriggers::ON_CALLS | HookTriggers::ON_RESUME, 
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Call => {
                    // This will be called for function calls
                    hook_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Resume => {
                    // This will be called when coroutine resumes
                    hook_counter.fetch_add(10, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        }
    )?;

    // Create a simple thread
    let thread = lua.create_thread(
        lua.load("coroutine.yield(42)")
        .into_function()?,
    )?;

    let initial_count = counter.load(Ordering::Relaxed);
    
    // Resume the thread - should trigger resume hook
    assert_eq!(thread.resume::<i32>(())?, 42);
    
    // Should have incremented by 10 for the resume event
    // (plus potentially some function call events)
    let final_count = counter.load(Ordering::Relaxed);
    assert!(final_count >= initial_count + 10);

    Ok(())
}
