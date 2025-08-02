#![cfg(all(feature = "async", not(feature = "luau")))]

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use mlua::{DebugEvent, HookTriggers, Lua, Result, VmState};
use futures_util::stream::StreamExt;

#[tokio::test]
async fn test_async_thread_resume_yield_hooks() -> Result<()> {
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
        },
    )?;

    // Test AsyncThread as Future with multiple yields
    let thread = lua.create_thread(
        lua.load(
            r#"
            coroutine.yield(1)
            coroutine.yield(2)
            return 42
        "#,
        )
        .into_function()?,
    )?;

    let async_thread = thread.into_async::<i32>(())?;
    let result = async_thread.await?;
    assert_eq!(result, 42);

    // Should have multiple resume and yield hooks triggered
    let resume_count = resume_counter.load(Ordering::Relaxed);
    let yield_count = yield_counter.load(Ordering::Relaxed);
    
    println!("Resume hooks: {}, Yield hooks: {}", resume_count, yield_count);
    
    // We expect:
    // - Multiple resumes (at least 3: initial + 2 after yields)
    // - Multiple yields (2 from coroutine.yield calls)
    assert!(resume_count >= 3);
    assert_eq!(yield_count, 2);

    Ok(())
}

#[tokio::test]
async fn test_async_thread_stream_hooks() -> Result<()> {
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
        },
    )?;

    // Test AsyncThread as Stream
    let thread = lua.create_thread(
        lua.load(
            r#"
            for i = 1, 3 do
                coroutine.yield(i)
            end
        "#,
        )
        .into_function()?,
    )?;

    let mut async_thread = thread.into_async::<i32>(())?;
    let mut results = Vec::new();
    
    // Collect yielded values
    while let Some(result) = async_thread.next().await {
        match result {
            Ok(value) => results.push(value),
            Err(_) => break, // Thread finished
        }
    }
    
    assert_eq!(results, vec![1, 2, 3]);

    // Check hook counts
    let resume_count = resume_counter.load(Ordering::Relaxed);
    let yield_count = yield_counter.load(Ordering::Relaxed);
    
    println!("Stream - Resume hooks: {}, Yield hooks: {}", resume_count, yield_count);
    
    // We expect multiple resumes and yields for the stream operations
    assert!(resume_count >= 3);
    assert_eq!(yield_count, 3); // 3 coroutine.yield calls

    Ok(())
}

#[tokio::test]
async fn test_async_thread_with_async_functions_hooks() -> Result<()> {
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
        },
    )?;

    // Create an async function that yields
    let async_func = lua.create_async_function(|_lua, n: i32| async move {
        // This will cause yield hooks when the async function awaits
        tokio::task::yield_now().await;
        Ok(n * 2)
    })?;
    lua.globals().set("async_func", async_func)?;

    // Test AsyncThread calling async functions
    let thread = lua.create_thread(
        lua.load(
            r#"
            local result1 = async_func(5)
            coroutine.yield(result1)
            local result2 = async_func(10)
            return result1 + result2
        "#,
        )
        .into_function()?,
    )?;

    let async_thread = thread.into_async::<i32>(())?;
    let result = async_thread.await?;
    assert_eq!(result, 30); // 5*2 + 10*2 = 30

    // Check that we got both types of yields:
    // - Coroutine yields from coroutine.yield()
    // - Async function yields from tokio::task::yield_now().await
    let resume_count = resume_counter.load(Ordering::Relaxed);
    let yield_count = yield_counter.load(Ordering::Relaxed);
    
    println!("Async+Coroutine - Resume hooks: {}, Yield hooks: {}", resume_count, yield_count);
    
    // We expect yields from both async functions and coroutine.yield
    assert!(resume_count >= 2);
    assert!(yield_count >= 3); // At least 1 coroutine.yield + 2 async function yields

    Ok(())
}
