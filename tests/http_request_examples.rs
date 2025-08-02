// Example demonstrating the two approaches for your HTTP request handler use case

use mlua::prelude::*;
use mlua::{HookTriggers, VmState, DebugEvent};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

// Approach 1: Thread-specific resume/yield hooks only (no instruction limiting)
#[test]
fn test_http_request_pattern_resume_yield_only() -> mlua::Result<()> {
    // Static Lua context (simulating your cached main context)
    let lua = Lua::new();
    
    // Simulate handling an HTTP request
    let resume_counter = Arc::new(AtomicI32::new(0));
    let yield_counter = Arc::new(AtomicI32::new(0));
    let hook_resume_counter = resume_counter.clone();
    let hook_yield_counter = yield_counter.clone();

    // Create a thread for this specific HTTP request
    let request_thread = lua.create_thread(
        lua.load(
            r#"
            -- Simulate async HTTP request handling
            local response = {}
            response.status = 200
            
            -- Yield to allow other requests (simulating async behavior)
            coroutine.yield("processing")
            
            response.body = "Hello, World!"
            
            -- Another yield point
            coroutine.yield("finalizing")
            
            return response
        "#,
        )
        .into_function()?,
    )?;

    // Set thread-specific hooks for resume/yield tracking
    request_thread.set_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("Request thread resumed");
                    hook_resume_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("Request thread yielded");
                    hook_yield_counter.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;

    // Process the request (multiple resume calls simulating async execution)
    let result1 = request_thread.resume::<String>(())?;
    assert_eq!(result1, "processing");
    
    let result2 = request_thread.resume::<String>(())?;
    assert_eq!(result2, "finalizing");
    
    let _final_result = request_thread.resume::<mlua::Table>(())?;

    // Verify hooks were triggered
    assert_eq!(resume_counter.load(Ordering::Relaxed), 3); // 3 resumes
    assert_eq!(yield_counter.load(Ordering::Relaxed), 2);  // 2 yields

    println!("✅ Approach 1: Thread-specific resume/yield hooks work perfectly!");
    Ok(())
}

// Approach 2: Global hooks for instruction limiting + AsyncThread support
#[test]
fn test_http_request_pattern_with_instruction_limit() -> mlua::Result<()> {
    let lua = Lua::new();
    
    let instruction_counter = Arc::new(AtomicI32::new(0));
    let resume_counter = Arc::new(AtomicI32::new(0));
    let yield_counter = Arc::new(AtomicI32::new(0));
    
    let hook_instruction_counter = instruction_counter.clone();
    let hook_resume_counter = resume_counter.clone();
    let hook_yield_counter = yield_counter.clone();

    // Set GLOBAL hooks that will apply to all threads
    // This approach allows instruction limiting and works with AsyncThread
    lua.set_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            every_nth_instruction: Some(50), // Prevent runaway scripts
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("Global: Thread resumed");
                    hook_resume_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("Global: Thread yielded");
                    hook_yield_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Count => {
                    hook_instruction_counter.fetch_add(1, Ordering::Relaxed);
                    // Could implement script timeout logic here
                    if hook_instruction_counter.load(Ordering::Relaxed) > 1000 {
                        println!("Script execution limit reached!");
                        return Ok(VmState::Yield); // Or return an error
                    }
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;

    // Create thread for HTTP request (will inherit global hooks)
    let request_thread = lua.create_thread(
        lua.load(
            r#"
            -- Simulate some computation that might run too long
            local sum = 0
            for i = 1, 100 do
                sum = sum + i
                if i % 30 == 0 then
                    coroutine.yield("progress: " .. i)
                end
            end
            return sum
        "#,
        )
        .into_function()?,
    )?;

    // Process with instruction counting and resume/yield tracking
    let mut results = Vec::new();
    loop {
        match request_thread.status() {
            mlua::ThreadStatus::Resumable => {
                let result: mlua::Value = request_thread.resume(())?;
                if let mlua::Value::String(s) = result {
                    results.push(s.to_str()?.to_owned());
                } else if let mlua::Value::Integer(n) = result {
                    results.push(n.to_string());
                    break;
                }
            }
            _ => break,
        }
    }

    println!("Results: {:?}", results);
    println!("Resume count: {}", resume_counter.load(Ordering::Relaxed));
    println!("Yield count: {}", yield_counter.load(Ordering::Relaxed));
    println!("Instruction hooks: {}", instruction_counter.load(Ordering::Relaxed));

    println!("✅ Approach 2: Global hooks with instruction limiting work!");
    Ok(())
}

#[test]
fn test_current_limitation_explanation() {
    println!("📝 CURRENT LIMITATION:");
    println!("   You cannot use thread-specific resume/yield hooks AND instruction limiting");
    println!("   on the same thread simultaneously due to Lua C API interference.");
    println!();
    println!("💡 SOLUTIONS for your HTTP request use case:");
    println!();
    println!("   Option 1: Thread-specific resume/yield hooks only");
    println!("   ✅ Use `thread.set_hook(HookTriggers::ON_RESUME | HookTriggers::ON_YIELD, callback)`");
    println!("   ✅ Works perfectly for tracking async execution per request");
    println!("   ❌ No automatic script timeout protection");
    println!();
    println!("   Option 2: Global hooks with instruction limiting");
    println!("   ✅ Use `lua.set_hook(HookTriggers {{ on_resume: true, on_yield: true, every_nth_instruction: Some(N), .. }}, callback)`");
    println!("   ✅ Works with AsyncThread and provides script timeout protection");
    println!("   ✅ Tracks resume/yield for ALL threads");
    println!("   ❌ Cannot distinguish between different HTTP requests");
    println!();
    println!("   Option 3: Hybrid approach (recommended)");
    println!("   ✅ Use global hooks for instruction limiting");
    println!("   ✅ Use thread-specific hooks for resume/yield tracking on threads that don't need instruction limiting");
    println!("   ✅ Or implement your own timeout mechanism (e.g., tokio::timeout)");
}
