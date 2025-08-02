#![cfg(not(feature = "luau"))]

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use mlua::{DebugEvent, HookTriggers, Lua, Result, ThreadStatus, VmState};

#[test]
fn test_thread_specific_resume_yield_hooks() -> Result<()> {
    let lua = Lua::new();

    let resume_counter = Arc::new(AtomicI32::new(0));
    let yield_counter = Arc::new(AtomicI32::new(0));
    let hook_resume_counter = resume_counter.clone();
    let hook_yield_counter = yield_counter.clone();

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

    // Set hooks on the specific thread (not globally)
    thread.set_hook(
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
        },
    )?;

    // First resume - should trigger resume hook and yield hook
    assert_eq!(thread.resume::<i32>(())?, 5);
    assert_eq!(thread.status(), ThreadStatus::Resumable);
    assert_eq!(resume_counter.load(Ordering::Relaxed), 1);
    assert_eq!(yield_counter.load(Ordering::Relaxed), 1);

    // Second resume - should trigger resume hook and yield hook again
    assert_eq!(thread.resume::<i32>(())?, 10);
    assert_eq!(thread.status(), ThreadStatus::Resumable);
    assert_eq!(resume_counter.load(Ordering::Relaxed), 2);
    assert_eq!(yield_counter.load(Ordering::Relaxed), 2);

    // Final resume - should trigger resume hook but no yield hook
    assert_eq!(thread.resume::<i32>(())?, 11);
    assert_eq!(thread.status(), ThreadStatus::Finished);
    assert_eq!(resume_counter.load(Ordering::Relaxed), 3);
    assert_eq!(yield_counter.load(Ordering::Relaxed), 2);

    Ok(())
}

#[test]
fn test_thread_specific_resume_hooks_only() -> Result<()> {
    let lua = Lua::new();

    let resume_counter = Arc::new(AtomicI32::new(0));
    let hook_resume_counter = resume_counter.clone();

    let thread = lua.create_thread(
        lua.load("coroutine.yield(42); return 24").into_function()?,
    )?;

    // Set only resume hooks on the thread
    thread.set_hook(HookTriggers::ON_RESUME, move |_lua, debug| {
        assert_eq!(debug.event(), DebugEvent::Resume);
        hook_resume_counter.fetch_add(1, Ordering::Relaxed);
        Ok(VmState::Continue)
    })?;

    // Should trigger resume hooks but not yield hooks (no global yield hooks set)
    assert_eq!(thread.resume::<i32>(())?, 42);
    assert_eq!(thread.resume::<i32>(())?, 24);
    
    assert_eq!(resume_counter.load(Ordering::Relaxed), 2);

    Ok(())
}

#[test]
fn test_thread_specific_yield_hooks_only() -> Result<()> {
    let lua = Lua::new();

    let yield_counter = Arc::new(AtomicI32::new(0));
    let hook_yield_counter = yield_counter.clone();

    let thread = lua.create_thread(
        lua.load("coroutine.yield(42); return 24").into_function()?,
    )?;

    // Set only yield hooks on the thread
    thread.set_hook(HookTriggers::ON_YIELD, move |_lua, debug| {
        assert_eq!(debug.event(), DebugEvent::Yield);
        hook_yield_counter.fetch_add(1, Ordering::Relaxed);
        Ok(VmState::Continue)
    })?;

    // Should trigger yield hooks but not resume hooks (no global resume hooks set)
    assert_eq!(thread.resume::<i32>(())?, 42);
    assert_eq!(thread.resume::<i32>(())?, 24);
    
    assert_eq!(yield_counter.load(Ordering::Relaxed), 1);

    Ok(())
}
