use mlua::{DebugEvent, HookTriggers, Lua, Result, ThreadStatus, VmState};

fn main() -> Result<()> {
    let lua = Lua::new();

    // Set a global hook that triggers when coroutines resume
    lua.set_global_hook(HookTriggers::ON_RESUME, |_lua, debug| {
        println!("Coroutine resumed! Event: {:?}", debug.event());
        
        // You can inspect debug information for the resume event
        let source = debug.source();
        println!("Source info: what={}, source={:?}", source.what, source.source);
        
        Ok(VmState::Continue)
    })?;

    // Create a coroutine that yields multiple times
    let thread = lua.create_thread(
        lua.load(
            r#"
            print("Coroutine starting...")
            coroutine.yield(1)
            print("Coroutine resumed first time")
            coroutine.yield(2)
            print("Coroutine resumed second time")
            return 3
        "#,
        )
        .into_function()?,
    )?;

    println!("First resume:");
    let result: i32 = thread.resume(())?;
    println!("Result: {}", result);
    assert_eq!(result, 1);
    assert_eq!(thread.status(), ThreadStatus::Resumable);

    println!("\nSecond resume:");
    let result: i32 = thread.resume(())?;
    println!("Result: {}", result);
    assert_eq!(result, 2);
    assert_eq!(thread.status(), ThreadStatus::Resumable);

    println!("\nThird resume:");
    let result: i32 = thread.resume(())?;
    println!("Result: {}", result);
    assert_eq!(result, 3);
    assert_eq!(thread.status(), ThreadStatus::Finished);

    println!("\nExample completed successfully!");
    Ok(())
}
