#![cfg(not(feature = "luau"))]

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use mlua::{DebugEvent, HookTriggers, Lua, Result, ThreadStatus, VmState};

#[test]
fn test_yield_hook() -> Result<()> {
    let lua = Lua::new();

    let counter = Arc::new(AtomicI32::new(0));
    let hook_counter = counter.clone();
    
    // Set a global hook that triggers on yield
    lua.set_global_hook(HookTriggers::ON_YIELD, move |_lua, debug| {
        // Verify this is a yield event
        assert_eq!(debug.event(), DebugEvent::Yield);
        
        // Verify source information for yield events
        let source = debug.source();
        assert_eq!(source.what, "yield");
        assert!(source.source.is_some());
        assert!(source.short_src.is_some());
        
        hook_counter.fetch_add(1, Ordering::Relaxed);
        Ok(VmState::Continue)
    })?;

    // Create a thread that will yield multiple times
    let thread = lua.create_thread(
        lua.load(
            r#"
            local x = 2 + 3
            coroutine.yield(x)
            local y = x * 2
            coroutine.yield(y)
            return y + 1
        "#,
        )
        .into_function()?,
    )?;

    // First resume - this should trigger the yield hook when coroutine.yield(x) is called
    assert_eq!(thread.resume::<i32>(())?, 5);
    assert_eq!(thread.status(), ThreadStatus::Resumable);
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    // Second resume - this should trigger the yield hook again when coroutine.yield(y) is called
    assert_eq!(thread.resume::<i32>(())?, 10);
    assert_eq!(thread.status(), ThreadStatus::Resumable);
    assert_eq!(counter.load(Ordering::Relaxed), 2);

    // Final resume - this should not trigger yield hook (function returns normally)
    assert_eq!(thread.resume::<i32>(())?, 11);
    assert_eq!(thread.status(), ThreadStatus::Finished);
    assert_eq!(counter.load(Ordering::Relaxed), 2);

    Ok(())
}

#[test]
fn test_yield_hook_combined() -> Result<()> {
    let lua = Lua::new();

    let counter = Arc::new(AtomicI32::new(0));
    let hook_counter = counter.clone();
    
    // Set a hook that triggers on both resume and yield
    lua.set_global_hook(
        HookTriggers::ON_RESUME | HookTriggers::ON_YIELD, 
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    // This will be called when coroutine resumes
                    hook_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    // This will be called when coroutine yields
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
    
    // Resume the thread - should trigger resume hook (1) and yield hook (10)
    assert_eq!(thread.resume::<i32>(())?, 42);
    
    // Should have incremented by 11 total (1 for resume + 10 for yield)
    let final_count = counter.load(Ordering::Relaxed);
    assert_eq!(final_count, initial_count + 11);

    Ok(())
}

#[test]
fn test_yield_and_resume_hooks_combined() -> Result<()> {
    let lua = Lua::new();

    let resume_counter = Arc::new(AtomicI32::new(0));
    let yield_counter = Arc::new(AtomicI32::new(0));
    let hook_resume_counter = resume_counter.clone();
    let hook_yield_counter = yield_counter.clone();
    
    // Set hooks for both resume and yield events
    lua.set_global_hook(
        HookTriggers::ON_RESUME | HookTriggers::ON_YIELD, 
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    hook_resume_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    hook_yield_counter.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        }
    )?;

    // Create a thread with multiple yield points
    let thread = lua.create_thread(
        lua.load(
            r#"
            coroutine.yield(1)
            coroutine.yield(2)
            coroutine.yield(3)
            return 4
        "#,
        )
        .into_function()?,
    )?;

    // First resume and yield
    assert_eq!(thread.resume::<i32>(())?, 1);
    assert_eq!(resume_counter.load(Ordering::Relaxed), 1);
    assert_eq!(yield_counter.load(Ordering::Relaxed), 1);

    // Second resume and yield
    assert_eq!(thread.resume::<i32>(())?, 2);
    assert_eq!(resume_counter.load(Ordering::Relaxed), 2);
    assert_eq!(yield_counter.load(Ordering::Relaxed), 2);

    // Third resume and yield
    assert_eq!(thread.resume::<i32>(())?, 3);
    assert_eq!(resume_counter.load(Ordering::Relaxed), 3);
    assert_eq!(yield_counter.load(Ordering::Relaxed), 3);

    // Final resume (no yield)
    assert_eq!(thread.resume::<i32>(())?, 4);
    assert_eq!(resume_counter.load(Ordering::Relaxed), 4);
    assert_eq!(yield_counter.load(Ordering::Relaxed), 3); // No additional yield

    Ok(())
}
