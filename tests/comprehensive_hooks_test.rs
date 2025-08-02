use mlua::prelude::*;
use mlua::{HookTriggers, VmState, DebugEvent};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

/// Comprehensive test showing native hooks and custom hooks working together
#[test]
fn test_comprehensive_hook_coexistence() -> mlua::Result<()> {
    let lua = Lua::new();
    
    // Counters for different hook types
    let line_count = Arc::new(AtomicI32::new(0));
    let call_count = Arc::new(AtomicI32::new(0));
    let return_count = Arc::new(AtomicI32::new(0));
    let resume_count = Arc::new(AtomicI32::new(0));
    let yield_count = Arc::new(AtomicI32::new(0));
    
    let line_counter = line_count.clone();
    let call_counter = call_count.clone();
    let return_counter = return_count.clone();
    let resume_counter = resume_count.clone();
    let yield_counter = yield_count.clone();
    
    // Create a thread with complex execution pattern
    let thread = lua.create_thread(
        lua.load(r#"
            local function helper(x)
                return x * 2
            end
            
            local a = 5
            local b = helper(a)
            coroutine.yield(b)
            local c = helper(b)
            return c
        "#).into_function()?,
    )?;
    
    // Set comprehensive hooks that include both native and custom triggers
    thread.set_hook(
        HookTriggers {
            every_line: true,       // Native hook
            on_calls: true,         // Native hook  
            on_returns: true,       // Native hook
            on_resume: true,        // Custom hook
            on_yield: true,         // Custom hook
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Line => {
                    println!("📍 Line: {:?}", debug.current_line());
                    line_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Call => {
                    println!("📞 Call: {:?}", debug.names());
                    call_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Ret => {
                    println!("↩️  Return: {:?}", debug.names());
                    return_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Resume => {
                    println!("▶️  Resume");
                    resume_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("⏸️  Yield");
                    yield_counter.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;
    
    println!("=== Testing comprehensive hook coexistence ===");
    
    // First resume - should trigger all types of hooks
    println!("\n--- First Resume ---");
    let result = thread.resume::<i32>(())?;
    println!("Yielded: {}", result);
    
    // Second resume - should trigger hooks again
    println!("\n--- Second Resume ---");
    let final_result = thread.resume::<i32>(())?;
    println!("Final result: {}", final_result);
    
    println!("\n--- Hook Summary ---");
    let line_total = line_count.load(Ordering::Relaxed);
    let call_total = call_count.load(Ordering::Relaxed);
    let return_total = return_count.load(Ordering::Relaxed);
    let resume_total = resume_count.load(Ordering::Relaxed);
    let yield_total = yield_count.load(Ordering::Relaxed);
    
    println!("📍 Line hooks: {}", line_total);
    println!("📞 Call hooks: {}", call_total);
    println!("↩️  Return hooks: {}", return_total);
    println!("▶️  Resume hooks: {}", resume_total);
    println!("⏸️  Yield hooks: {}", yield_total);
    
    // Verify that all hook types fired
    assert!(line_total > 0, "Line hooks should have fired");
    assert!(call_total > 0, "Call hooks should have fired");
    assert!(return_total > 0, "Return hooks should have fired");
    assert_eq!(resume_total, 2, "Should have exactly 2 resume hooks");
    assert_eq!(yield_total, 1, "Should have exactly 1 yield hook");
    
    println!("\n✅ All hook types working together successfully!");
    
    Ok(())
}
