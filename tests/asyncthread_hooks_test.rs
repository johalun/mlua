use mlua::prelude::*;
use mlua::{HookTriggers, VmState, DebugEvent};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use futures_util::StreamExt;

#[tokio::test]
async fn test_asyncthread_with_global_hooks() -> mlua::Result<()> {
    let lua = Lua::new();
    
    let resume_counter = Arc::new(AtomicI32::new(0));
    let yield_counter = Arc::new(AtomicI32::new(0));
    let instruction_counter = Arc::new(AtomicI32::new(0));
    
    let hook_resume_counter = resume_counter.clone();
    let hook_yield_counter = yield_counter.clone();
    let hook_instruction_counter = instruction_counter.clone();

    // Set global hooks that work with AsyncThread
    lua.set_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            every_nth_instruction: Some(20),
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("AsyncThread resumed");
                    hook_resume_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("AsyncThread yielded");
                    hook_yield_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Count => {
                    hook_instruction_counter.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;

    // Create an AsyncThread (works with global hooks)
    let thread = lua.create_thread(
        lua.load(
            r#"
            -- Simulate async work that yields
            local result = {}
            for i = 1, 3 do
                local data = coroutine.yield("fetch_data_" .. i)
                table.insert(result, data or ("default_" .. i))
            end
            return result
        "#,
        )
        .into_function()?,
    )?;
    
    let mut async_thread = thread.into_async::<mlua::Value>(())?;

    // Process the AsyncThread as a Stream
    let mut responses = Vec::new();
    while let Some(result) = async_thread.next().await {
        match result? {
            mlua::Value::String(s) => {
                let yielded = s.to_str()?;
                println!("AsyncThread yielded: {}", yielded);
                responses.push(yielded.to_owned());
                // We can't easily send data back in this simplified test
                // but the hooks should still be triggered
            }
            mlua::Value::Table(_) => {
                println!("AsyncThread completed with final result");
                break;
            }
            _ => {}
        }
    }

    println!("Resume count: {}", resume_counter.load(Ordering::Relaxed));
    println!("Yield count: {}", yield_counter.load(Ordering::Relaxed));
    println!("Instruction count: {}", instruction_counter.load(Ordering::Relaxed));

    // Verify that hooks were triggered
    assert!(resume_counter.load(Ordering::Relaxed) > 0);
    assert!(yield_counter.load(Ordering::Relaxed) > 0);

    println!("✅ AsyncThread works perfectly with global resume/yield hooks!");
    Ok(())
}
