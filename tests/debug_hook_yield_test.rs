use mlua::prelude::*;
use mlua::{HookTriggers, VmState, ThreadStatus};

#[test]
fn debug_hook_yield() -> mlua::Result<()> {
    let lua = Lua::new();

    let func = lua
        .load(
            r#"
            local x = 2 + 3
            local y = x * 63
            local z = string.len(x..", "..y)
        "#,
        )
        .into_function()?;
    let co = lua.create_thread(func)?;

    co.set_hook(HookTriggers::EVERY_LINE, move |_lua, debug| {
        println!("Hook triggered on line {:?}, yielding...", debug.current_line());
        Ok(VmState::Yield)
    })?;

    println!("=== Testing exact same pattern as original test ===");
    
    println!("First resume:");
    let result1 = co.resume::<()>(());
    println!("  Result: {:?}", result1);
    println!("  Status: {:?}", co.status());
    
    println!("Second resume:");
    let result2 = co.resume::<()>(());
    println!("  Result: {:?}", result2);
    println!("  Status: {:?}", co.status());
    
    println!("Third resume:");
    let result3 = co.resume::<()>(());
    println!("  Result: {:?}", result3);
    println!("  Status: {:?}", co.status());
    
    println!("Fourth resume:");
    let result4 = co.resume::<()>(());
    println!("  Result: {:?}", result4);
    println!("  Status: {:?}", co.status());
    
    println!("Fifth resume (if needed):");
    if co.status() == ThreadStatus::Resumable {
        let result5 = co.resume::<()>(());
        println!("  Result: {:?}", result5);
        println!("  Status: {:?}", co.status());
    }
    
    Ok(())
}
