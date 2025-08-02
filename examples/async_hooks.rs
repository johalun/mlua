use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use mlua::{DebugEvent, HookTriggers, Lua, Result, VmState};

#[tokio::main]
async fn main() -> Result<()> {
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
                }
                DebugEvent::Yield => {
                    let count = hook_yield_counter.fetch_add(1, Ordering::Relaxed) + 1;
                    println!("⏸️  Yield hook triggered (count: {})", count);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;

    println!("=== Demonstration: Async Function Yield Hooks ===\n");

    // Create an async function that will yield when it awaits
    let async_func = lua.create_async_function(|_lua, name: String| async move {
        println!("Async function: Starting with name: {}", name);
        
        // This await will cause the async function to yield
        tokio::task::yield_now().await;
        
        println!("Async function: Resumed after yield");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        println!("Async function: Completed processing");
        Ok(format!("Hello, {}!", name))
    })?;
    
    lua.globals().set("async_func", async_func)?;

    println!("📋 Created async function. Calling it...\n");

    // Call the async function - this should trigger yield hooks when it yields
    let result: String = lua.load(r#"return async_func("World")"#).eval_async().await?;
    println!("✅ Async function returned: {}\n", result);

    // Also test regular coroutine hooks
    println!("📋 Testing regular coroutine as well...\n");
    
    let thread = lua.create_thread(
        lua.load("coroutine.yield(42); return 'done'").into_function()?,
    )?;

    println!("🔄 Resuming coroutine...");
    let yield_result: i32 = thread.resume(())?;
    println!("✅ Coroutine yielded: {}", yield_result);

    println!("🔄 Resuming coroutine again...");
    let final_result: String = thread.resume(())?;
    println!("✅ Coroutine finished: {}\n", final_result);

    // Print summary
    println!("=== Summary ===");
    println!("Resume hooks triggered: {}", resume_counter.load(Ordering::Relaxed));
    println!("Yield hooks triggered: {}", yield_counter.load(Ordering::Relaxed));
    println!("\n🎉 Both coroutine and async function hooks are working!");

    Ok(())
}
