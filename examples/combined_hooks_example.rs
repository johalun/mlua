use mlua::{DebugEvent, HookTriggers, Lua, Result, ThreadStatus, VmState};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

fn main() -> Result<()> {
    let lua = Lua::new();

    let call_counter = Arc::new(AtomicI32::new(0));
    let resume_counter = Arc::new(AtomicI32::new(0));
    
    let call_counter_clone = call_counter.clone();
    let resume_counter_clone = resume_counter.clone();

    // Set a hook that triggers on both function calls and coroutine resumes
    lua.set_global_hook(
        HookTriggers::ON_CALLS | HookTriggers::ON_RESUME,
        move |_lua, debug| {
            match debug.event() {
                DebugEvent::Call => {
                    call_counter_clone.fetch_add(1, Ordering::Relaxed);
                    let names = debug.names();
                    println!("Function call: {:?}", names.name);
                }
                DebugEvent::Resume => {
                    resume_counter_clone.fetch_add(1, Ordering::Relaxed);
                    println!("Coroutine resume event detected!");
                }
                _ => {}
            }
            Ok(VmState::Continue)
        },
    )?;

    println!("Creating a coroutine that calls functions...");
    
    // Create a coroutine that calls functions and yields
    let thread = lua.create_thread(
        lua.load(
            r#"
            function helper()
                return "helper called"
            end
            
            print("Starting coroutine")
            local msg = helper()
            coroutine.yield(msg)
            
            local len = string.len("test")
            return len
        "#,
        )
        .into_function()?,
    )?;

    println!("\nResuming coroutine first time:");
    let result: String = thread.resume(())?;
    println!("First resume result: {}", result);

    println!("\nResuming coroutine second time:");
    let result: i32 = thread.resume(())?;
    println!("Second resume result: {}", result);

    println!("\nSummary:");
    println!("Function calls detected: {}", call_counter.load(Ordering::Relaxed));
    println!("Resume events detected: {}", resume_counter.load(Ordering::Relaxed));
    println!("Thread status: {:?}", thread.status());

    Ok(())
}
