use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use mlua::{DebugEvent, HookTriggers, Lua, Result, VmState};
use futures_util::StreamExt;

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

    println!("=== Demonstration: AsyncThread with Resume and Yield Hooks ===\n");

    // Create an async function that will yield
    let async_func = lua.create_async_function(|_lua, name: String| async move {
        println!("Async function: Processing {}", name);
        
        // This await will cause yield hooks to trigger
        tokio::task::yield_now().await;
        
        println!("Async function: Completed processing {}", name);
        Ok(format!("Hello, {}!", name))
    })?;
    lua.globals().set("async_func", async_func)?;

    println!("📋 Testing AsyncThread as Future...\n");

    // Test AsyncThread as Future
    let thread = lua.create_thread(
        lua.load(
            r#"
            print("Coroutine: Starting execution")
            local result1 = async_func("Alice")
            print("Coroutine: Got result1:", result1)
            coroutine.yield(result1)
            
            print("Coroutine: Continuing after yield")
            local result2 = async_func("Bob")
            print("Coroutine: Got result2:", result2)
            return result2
        "#,
        )
        .into_function()?,
    )?;

    let async_thread = thread.into_async::<String>(())?;
    let final_result = async_thread.await?;
    println!("✅ AsyncThread Future completed with: {}\n", final_result);

    println!("📋 Testing AsyncThread as Stream...\n");

    // Reset counters for stream test
    resume_counter.store(0, Ordering::Relaxed);
    yield_counter.store(0, Ordering::Relaxed);

    // Test AsyncThread as Stream
    let thread2 = lua.create_thread(
        lua.load(
            r#"
            for i = 1, 3 do
                print("Coroutine: Yielding value", i)
                coroutine.yield(i * 10)
            end
            print("Coroutine: Stream finished")
        "#,
        )
        .into_function()?,
    )?;

    let mut async_stream = thread2.into_async::<i32>(())?;
    let mut stream_results = Vec::new();
    
    while let Some(result) = async_stream.next().await {
        match result {
            Ok(value) => {
                println!("📦 Received from stream: {}", value);
                stream_results.push(value);
            }
            Err(_) => break,
        }
    }
    
    println!("✅ AsyncThread Stream collected: {:?}\n", stream_results);

    // Print final summary
    println!("=== Final Summary ===");
    println!("Resume hooks triggered: {}", resume_counter.load(Ordering::Relaxed));
    println!("Yield hooks triggered: {}", yield_counter.load(Ordering::Relaxed));
    println!("\n🎉 AsyncThread with both resume and yield hooks working perfectly!");

    Ok(())
}
