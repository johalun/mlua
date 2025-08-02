use mlua::prelude::*;
use mlua::{HookTriggers, VmState, DebugEvent};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

/// Test that simulates the user's HTTP server scenario:
/// - Static Lua context (cache)  
/// - Per-request threads
/// - Thread-specific hooks for request monitoring
#[test]
fn test_http_server_scenario() -> mlua::Result<()> {
    // Static Lua context - like a cache that persists across requests
    let lua = Lua::new();
    
    // Set up some global state in the cache
    lua.globals().set("cache_data", "shared_value")?;
    
    // Request counters for monitoring
    let request1_resume_count = Arc::new(AtomicI32::new(0));
    let request1_yield_count = Arc::new(AtomicI32::new(0));
    let request2_resume_count = Arc::new(AtomicI32::new(0));
    let request2_yield_count = Arc::new(AtomicI32::new(0));
    
    println!("=== Simulating HTTP Server with Per-Request Threads ===");
    
    // REQUEST 1: Create a thread for first HTTP request
    let request1_thread = lua.create_thread(
        lua.load(r#"
            local cache = cache_data  -- Access shared cache
            coroutine.yield("processing_request_1_with_" .. cache)
            return "request_1_complete"
        "#).into_function()?,
    )?;
    
    // Set monitoring hooks for this specific request
    let req1_resume = request1_resume_count.clone();
    let req1_yield = request1_yield_count.clone();
    request1_thread.set_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("📈 Request 1 resumed");
                    req1_resume.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("⏸️  Request 1 yielded");
                    req1_yield.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;
    
    // REQUEST 2: Create a thread for second HTTP request (concurrent)
    let request2_thread = lua.create_thread(
        lua.load(r#"
            local cache = cache_data  -- Access same shared cache
            coroutine.yield("processing_request_2_with_" .. cache)
            return "request_2_complete"
        "#).into_function()?,
    )?;
    
    // Set different monitoring hooks for this request
    let req2_resume = request2_resume_count.clone();
    let req2_yield = request2_yield_count.clone();
    request2_thread.set_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("📈 Request 2 resumed");
                    req2_resume.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("⏸️  Request 2 yielded");
                    req2_yield.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;
    
    // Execute request 1 
    println!("\n--- Processing Request 1 ---");
    let result1 = request1_thread.resume::<String>(())?;
    println!("Request 1 yielded: {}", result1);
    let final1 = request1_thread.resume::<String>(())?;
    println!("Request 1 completed: {}", final1);
    
    // Execute request 2
    println!("\n--- Processing Request 2 ---");
    let result2 = request2_thread.resume::<String>(())?;
    println!("Request 2 yielded: {}", result2);
    let final2 = request2_thread.resume::<String>(())?;
    println!("Request 2 completed: {}", final2);
    
    // Verify each request was monitored independently
    println!("\n--- Hook Monitoring Results ---");
    println!("Request 1: {} resumes, {} yields", 
             request1_resume_count.load(Ordering::Relaxed),
             request1_yield_count.load(Ordering::Relaxed));
    println!("Request 2: {} resumes, {} yields", 
             request2_resume_count.load(Ordering::Relaxed),
             request2_yield_count.load(Ordering::Relaxed));
    
    // Each request should have exactly 2 resumes and 1 yield
    assert_eq!(request1_resume_count.load(Ordering::Relaxed), 2);
    assert_eq!(request1_yield_count.load(Ordering::Relaxed), 1);
    assert_eq!(request2_resume_count.load(Ordering::Relaxed), 2);
    assert_eq!(request2_yield_count.load(Ordering::Relaxed), 1);
    
    println!("✅ HTTP server scenario test passed!");
    
    Ok(())
}
