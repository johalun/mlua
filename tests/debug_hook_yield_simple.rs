use mlua::prelude::*;
use mlua::{HookTriggers, VmState, ThreadStatus};

#[test]
fn debug_hook_yield_simple() -> mlua::Result<()> {
    let lua = Lua::new();

    // Test with simpler Lua code first
    let func = lua
        .load(
            r#"
            local x = 2 + 3
            local y = x * 63
            return y
        "#,
        )
        .into_function()?;
    let co = lua.create_thread(func)?;

    co.set_hook(HookTriggers::EVERY_LINE, move |_lua, debug| {
        println!("Hook on line {:?}", debug.current_line());
        Ok(VmState::Yield)
    })?;

    println!("=== Testing simplified Lua code ===");
    
    let mut resume_count = 0;
    loop {
        resume_count += 1;
        println!("Resume #{}", resume_count);
        
        match co.resume::<mlua::Value>(()) {
            Ok(value) => {
                println!("  Success: {:?}", value);
                println!("  Status: {:?}", co.status());
                if co.status() == ThreadStatus::Finished {
                    break;
                }
            }
            Err(e) => {
                println!("  Error: {:?}", e);
                println!("  Status: {:?}", co.status());
                break;
            }
        }
        
        if resume_count > 10 {
            println!("Too many resumes, breaking");
            break;
        }
    }
    
    Ok(())
}
