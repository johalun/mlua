use mlua::prelude::*;
use mlua::{HookTriggers, VmState, DebugEvent};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

#[test]
fn test_thread_hooks_with_instruction_limit() -> mlua::Result<()> {
    let lua = Lua::new();

    let resume_counter = Arc::new(AtomicI32::new(0));
    let yield_counter = Arc::new(AtomicI32::new(0));
    let instruction_counter = Arc::new(AtomicI32::new(0));
    
    let hook_resume_counter = resume_counter.clone();
    let hook_yield_counter = yield_counter.clone();
    let hook_instruction_counter = instruction_counter.clone();

    // Create a thread that will yield multiple times
    let thread = lua.create_thread(
        lua.load(
            r#"
            local x = 2 + 3
            coroutine.yield(x)
            local y = x * 2
            coroutine.yield(y)
            return y + 1
        "#,
        )
        .into_function()?,
    )?;

    // Set hooks combining custom triggers (resume/yield) with native triggers (every_nth_instruction)
    thread.set_hook(
        HookTriggers {
            on_resume: true,
            on_yield: true,
            every_nth_instruction: Some(10), // Trigger every 10 instructions
            ..Default::default()
        },
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Resume => {
                    println!("Resume hook triggered");
                    hook_resume_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Yield => {
                    println!("Yield hook triggered");
                    hook_yield_counter.fetch_add(1, Ordering::Relaxed);
                }
                DebugEvent::Count => {
                    println!("Instruction count hook triggered");
                    hook_instruction_counter.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;

    println!("Starting first resume...");
    assert_eq!(thread.resume::<i32>(())?, 5);
    println!("First resume completed, status: {:?}", thread.status());
    
    println!("Resume counter: {}", resume_counter.load(Ordering::Relaxed));
    println!("Yield counter: {}", yield_counter.load(Ordering::Relaxed));
    println!("Instruction counter: {}", instruction_counter.load(Ordering::Relaxed));

    println!("Starting second resume...");
    assert_eq!(thread.resume::<i32>(())?, 10);
    println!("Second resume completed, status: {:?}", thread.status());
    
    println!("Resume counter: {}", resume_counter.load(Ordering::Relaxed));
    println!("Yield counter: {}", yield_counter.load(Ordering::Relaxed));
    println!("Instruction counter: {}", instruction_counter.load(Ordering::Relaxed));

    println!("Starting final resume...");
    assert_eq!(thread.resume::<i32>(())?, 11);
    println!("Final resume completed, status: {:?}", thread.status());
    
    println!("Final counters:");
    println!("Resume counter: {}", resume_counter.load(Ordering::Relaxed));
    println!("Yield counter: {}", yield_counter.load(Ordering::Relaxed));
    println!("Instruction counter: {}", instruction_counter.load(Ordering::Relaxed));

    // Should have triggered resume hooks
    assert!(resume_counter.load(Ordering::Relaxed) > 0);
    // Should have triggered yield hooks (2 yields in the script)
    assert_eq!(yield_counter.load(Ordering::Relaxed), 2);
    // Should have triggered some instruction count hooks
    assert!(instruction_counter.load(Ordering::Relaxed) > 0);

    Ok(())
}
