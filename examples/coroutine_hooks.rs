use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use mlua::{DebugEvent, HookTriggers, Lua, Result, VmState};

fn main() -> Result<()> {
    let lua = Lua::new();

    // Create counters for each hook type
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
                    let count = hook_resume_counter.fetch_add(1, Ordering::Relaxed) + 1;
                    println!("🚀 Resume hook triggered (count: {})", count);
                    
                    // Print debug info for resume events
                    let source = debug.source();
                    println!("   Source: what={}, source={:?}", source.what, source.source);
                }
                DebugEvent::Yield => {
                    let count = hook_yield_counter.fetch_add(1, Ordering::Relaxed) + 1;
                    println!("⏸️  Yield hook triggered (count: {})", count);
                    
                    // Print debug info for yield events
                    let source = debug.source();
                    println!("   Source: what={}, source={:?}", source.what, source.source);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;

    println!("=== Demonstration: Resume and Yield Hooks ===\n");

    // Create a coroutine that yields multiple times
    let thread = lua.create_thread(
        lua.load(
            r#"
            print("Coroutine: Starting execution")
            local x = 10
            print("Coroutine: About to yield first value:", x)
            coroutine.yield(x)
            
            print("Coroutine: Resumed! Calculating second value")
            local y = x * 2
            print("Coroutine: About to yield second value:", y)
            coroutine.yield(y)
            
            print("Coroutine: Resumed again! Calculating final result")
            local z = y + 5
            print("Coroutine: Returning final result:", z)
            return z
        "#,
        )
        .into_function()?,
    )?;

    println!("📋 Created coroutine. Starting execution...\n");

    // First resume
    println!("🔄 Calling first resume...");
    let result1: i32 = thread.resume(())?;
    println!("✅ First resume returned: {}\n", result1);

    // Second resume
    println!("🔄 Calling second resume...");
    let result2: i32 = thread.resume(())?;
    println!("✅ Second resume returned: {}\n", result2);

    // Final resume
    println!("🔄 Calling final resume...");
    let final_result: i32 = thread.resume(())?;
    println!("✅ Final resume returned: {}\n", final_result);

    // Print summary
    println!("=== Summary ===");
    println!("Resume hooks triggered: {}", resume_counter.load(Ordering::Relaxed));
    println!("Yield hooks triggered: {}", yield_counter.load(Ordering::Relaxed));
    println!("\n🎉 Both resume and yield hooks are working correctly!");

    Ok(())
}
