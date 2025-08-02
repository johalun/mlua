use mlua::prelude::*;
use mlua::{HookTriggers, VmState, DebugEvent};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

/// Test to explore if native hooks and custom hooks can coexist safely
#[test]
fn test_native_and_custom_hooks_coexistence() -> mlua::Result<()> {
    let lua = Lua::new();
    
    let native_hook_count = Arc::new(AtomicI32::new(0));
    let custom_resume_count = Arc::new(AtomicI32::new(0));
    let custom_yield_count = Arc::new(AtomicI32::new(0));
    
    let native_counter1 = native_hook_count.clone();
    let native_counter2 = native_hook_count.clone();
    let custom_resume = custom_resume_count.clone();
    let custom_yield = custom_yield_count.clone();
    
    // Create a thread that will execute multiple lines and yield
    let thread = lua.create_thread(
        lua.load(r#"
            local x = 1  -- line 1
            local y = 2  -- line 2
            coroutine.yield(x + y)  -- line 3
            local z = 3  -- line 4
            return z + x + y  -- line 5
        "#).into_function()?,
    )?;
    
    // Set native hooks on the thread (every line)
    thread.set_hook(
        HookTriggers {
            every_line: true,
            ..Default::default()
        },
        move |_lua, debug| {
            if debug.event() == DebugEvent::Line {
                println!("🔧 Native hook: Line {:?}", debug.current_line());
                native_counter1.fetch_add(1, Ordering::Relaxed);
            }
            Ok(VmState::Continue)
        },
    )?;
    
    // Also set custom resume/yield hooks on the same thread
    // This should currently be prevented, but let's see what happens
    thread.set_hook(
        HookTriggers {
            every_line: true,  // Keep existing native hook
            on_resume: true,   // Add custom hook
            on_yield: true,    // Add custom hook
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Line => {
                    println!("🔧 Native hook: Line {:?}", debug.current_line());
                    native_counter2.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Resume => {
                    println!("📈 Custom resume hook");
                    custom_resume.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("⏸️  Custom yield hook");
                    custom_yield.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;
    
    println!("=== Testing native + custom hooks coexistence ===");
    
    // First resume - should trigger both native line hooks and custom resume hook
    println!("First resume...");
    let result = thread.resume::<i32>(())?;
    println!("Yielded: {}", result);
    
    // Second resume - should trigger native hooks and custom resume hook again
    println!("Second resume...");
    let final_result = thread.resume::<i32>(())?;
    println!("Final result: {}", final_result);
    
    println!("\n--- Hook Counts ---");
    println!("Native hooks (line): {}", native_hook_count.load(Ordering::Relaxed));
    println!("Custom resume hooks: {}", custom_resume_count.load(Ordering::Relaxed));
    println!("Custom yield hooks: {}", custom_yield_count.load(Ordering::Relaxed));
    
    // Check if both types of hooks fired
    let native_count = native_hook_count.load(Ordering::Relaxed);
    let resume_count = custom_resume_count.load(Ordering::Relaxed);
    let yield_count = custom_yield_count.load(Ordering::Relaxed);
    
    println!("Native hooks fired: {}", native_count > 0);
    println!("Custom resume hooks fired: {}", resume_count > 0);
    println!("Custom yield hooks fired: {}", yield_count > 0);
    
    // For now, this test is exploratory - let's see what actually happens
    Ok(())
}
