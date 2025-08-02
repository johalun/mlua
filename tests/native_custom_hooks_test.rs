use mlua::prelude::*;
use mlua::{HookTriggers, VmState, DebugEvent};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

/// Test demonstrating that native hooks and custom hooks work together perfectly
#[test]
fn test_native_and_custom_hooks_coexistence() -> mlua::Result<()> {
    let lua = Lua::new();
    
    let native_hook_count = Arc::new(AtomicI32::new(0));
    let custom_resume_count = Arc::new(AtomicI32::new(0));
    let custom_yield_count = Arc::new(AtomicI32::new(0));
    
    let native_counter = native_hook_count.clone();
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
    
    // Set both native hooks (every line) and custom hooks (resume/yield) on the same thread
    thread.set_hook(
        HookTriggers {
            every_line: true,  // Native hook
            on_resume: true,   // Custom hook
            on_yield: true,    // Custom hook
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Line => {
                    println!("🔧 Native hook: Line {:?}", debug.current_line());
                    native_counter.fetch_add(1, Ordering::Relaxed);
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
    
    // First resume - triggers both native line hooks and custom resume hook
    println!("First resume...");
    let result = thread.resume::<i32>(())?;
    println!("Yielded: {}", result);
    
    // Second resume - triggers native hooks and custom resume hook again
    println!("Second resume...");
    let final_result = thread.resume::<i32>(())?;
    println!("Final result: {}", final_result);
    
    println!("\n--- Hook Counts ---");
    println!("Native hooks (line): {}", native_hook_count.load(Ordering::Relaxed));
    println!("Custom resume hooks: {}", custom_resume_count.load(Ordering::Relaxed));
    println!("Custom yield hooks: {}", custom_yield_count.load(Ordering::Relaxed));
    
    // Verify that both native and custom hooks fired successfully
    let native_count = native_hook_count.load(Ordering::Relaxed);
    let resume_count = custom_resume_count.load(Ordering::Relaxed);
    let yield_count = custom_yield_count.load(Ordering::Relaxed);
    
    assert!(native_count > 0, "Native line hooks should have fired");
    assert_eq!(resume_count, 2, "Should have exactly 2 resume hooks");
    assert_eq!(yield_count, 1, "Should have exactly 1 yield hook");
    
    println!("\n✅ Native and custom hooks coexist perfectly!");
    
    Ok(())
}
