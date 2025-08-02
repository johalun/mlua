use mlua::{prelude::*, Debug};
use mlua::{HookTriggers, VmState, DebugEvent};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

/// Test to verify that global hooks and native hooks can coexist
#[test]
fn test_global_hooks_with_native_hooks() -> mlua::Result<()> {
    let lua = Lua::new();
    
    let global_resume_count = Arc::new(AtomicI32::new(0));
    let global_yield_count = Arc::new(AtomicI32::new(0));
    let native_line_count = Arc::new(AtomicI32::new(0));
    
    let global_resume = global_resume_count.clone();
    let global_yield = global_yield_count.clone();
    let native_line = native_line_count.clone();
    
    // Set GLOBAL resume/yield hooks
    lua.set_global_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            every_line: true,
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("🌍 Global resume hook");
                    global_resume.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Line => {
                    println!("📍 Native line hook: {:?}", debug.current_line());
                    native_line.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("🌍 Global yield hook");
                    global_yield.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;
    
    // Create a thread with native hooks (line triggers)
    let thread = lua.create_thread(
        lua.load(r#"
            local x = 1  -- line 1
            local y = 2  -- line 2
            coroutine.yield(x + y)  -- line 3
            local z = 3  -- line 4
            return z + x + y  -- line 5
        "#).into_function()?,
    )?;
    
    println!("=== Testing global hooks + native hooks coexistence ===");
    
    // First resume - should trigger both global resume hook and native line hooks
    println!("\n--- First Resume ---");
    let result = thread.resume::<i32>(())?;
    println!("Yielded: {}", result);
    
    // Second resume - should trigger global resume hook and more native line hooks
    println!("\n--- Second Resume ---");
    let final_result = thread.resume::<i32>(())?;
    println!("Final result: {}", final_result);
    
    println!("\n--- Hook Summary ---");
    let global_resume_total = global_resume_count.load(Ordering::Relaxed);
    let global_yield_total = global_yield_count.load(Ordering::Relaxed);
    let native_line_total = native_line_count.load(Ordering::Relaxed);
    
    println!("🌍 Global resume hooks: {}", global_resume_total);
    println!("🌍 Global yield hooks: {}", global_yield_total);
    println!("📍 Native line hooks: {}", native_line_total);
    
    // Verify that both global hooks and native hooks fired
    assert!(global_resume_total > 0, "Global resume hooks should have fired");
    assert!(global_yield_total > 0, "Global yield hooks should have fired");
    assert!(native_line_total > 0, "Native line hooks should have fired");
    
    println!("\n✅ Global hooks and native hooks working together!");
    
    Ok(())
}
