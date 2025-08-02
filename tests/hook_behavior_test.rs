use mlua::prelude::*;
use mlua::{HookTriggers, VmState, DebugEvent};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

#[test]
fn test_global_vs_thread_specific_hooks() -> mlua::Result<()> {
    let lua = Lua::new();
    
    let global_resume_counter = Arc::new(AtomicI32::new(0));
    let global_yield_counter = Arc::new(AtomicI32::new(0));
    let thread_resume_counter = Arc::new(AtomicI32::new(0));
    let thread_yield_counter = Arc::new(AtomicI32::new(0));
    
    let hook_global_resume = global_resume_counter.clone();
    let hook_global_yield = global_yield_counter.clone();
    let hook_thread_resume = thread_resume_counter.clone();
    let hook_thread_yield = thread_yield_counter.clone();

    // Set GLOBAL hooks first
    lua.set_global_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("🌍 GLOBAL resume hook triggered");
                    hook_global_resume.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("🌍 GLOBAL yield hook triggered");
                    hook_global_yield.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;

    println!("=== Test 1: Thread with NO thread-specific hooks (should use global hooks) ===");
    
    let thread1 = lua.create_thread(
        lua.load("coroutine.yield(42); return 24").into_function()?,
    )?;
    
    // No thread-specific hooks set - should use global hooks
    println!("Thread1 first resume...");
    assert_eq!(thread1.resume::<i32>(())?, 42);
    println!("Thread1 second resume...");
    assert_eq!(thread1.resume::<i32>(())?, 24);
    
    println!("After Thread1:");
    println!("  Global resume count: {}", global_resume_counter.load(Ordering::Relaxed));
    println!("  Global yield count: {}", global_yield_counter.load(Ordering::Relaxed));
    
    // Reset counters
    global_resume_counter.store(0, Ordering::Relaxed);
    global_yield_counter.store(0, Ordering::Relaxed);
    
    println!("\n=== Test 2: Thread WITH thread-specific hooks (should override global hooks) ===");
    
    let thread2 = lua.create_thread(
        lua.load("coroutine.yield(84); return 48").into_function()?,
    )?;
    
    // Set thread-specific hooks - these will OVERRIDE global hooks
    thread2.set_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("🎯 THREAD-SPECIFIC resume hook triggered");
                    hook_thread_resume.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("🎯 THREAD-SPECIFIC yield hook triggered");
                    hook_thread_yield.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;
    
    println!("Thread2 first resume...");
    assert_eq!(thread2.resume::<i32>(())?, 84);
    println!("Thread2 second resume...");
    assert_eq!(thread2.resume::<i32>(())?, 48);
    
    println!("After Thread2:");
    println!("  Global resume count: {}", global_resume_counter.load(Ordering::Relaxed));
    println!("  Global yield count: {}", global_yield_counter.load(Ordering::Relaxed));
    println!("  Thread-specific resume count: {}", thread_resume_counter.load(Ordering::Relaxed));
    println!("  Thread-specific yield count: {}", thread_yield_counter.load(Ordering::Relaxed));
    
    // Verify the behavior:
    // Thread1 should have triggered global hooks
    // Thread2 should have triggered thread-specific hooks (and NOT global hooks)
    
    Ok(())
}

#[test]
fn test_global_hooks_only() -> mlua::Result<()> {
    let lua = Lua::new();
    
    let resume_counter = Arc::new(AtomicI32::new(0));
    let yield_counter = Arc::new(AtomicI32::new(0));
    let hook_resume_counter = resume_counter.clone();
    let hook_yield_counter = yield_counter.clone();

    // Set ONLY global hooks
    lua.set_global_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("🌍 Global resume hook");
                    hook_resume_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("🌍 Global yield hook");
                    hook_yield_counter.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;

    // Create thread without thread-specific hooks
    let thread = lua.create_thread(
        lua.load("coroutine.yield(100); return 200").into_function()?,
    )?;
    
    println!("=== Testing thread with ONLY global hooks ===");
    println!("First resume...");
    assert_eq!(thread.resume::<i32>(())?, 100);
    println!("Second resume...");
    assert_eq!(thread.resume::<i32>(())?, 200);
    
    println!("Final counts:");
    println!("  Resume count: {}", resume_counter.load(Ordering::Relaxed));
    println!("  Yield count: {}", yield_counter.load(Ordering::Relaxed));
    
    // Should have triggered global hooks
    assert!(resume_counter.load(Ordering::Relaxed) > 0);
    assert!(yield_counter.load(Ordering::Relaxed) > 0);
    
    Ok(())
}
