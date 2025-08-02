use mlua::prelude::*;
use mlua::{HookTriggers, VmState};

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

    co.set_hook(HookTriggers::EVERY_LINE, move |_lua, _debug| Ok(VmState::Yield))?;

    println!("Starting resumes...");
    match co.resume::<()>(()) {
        Ok(_) => println!("First resume: OK"),
        Err(e) => println!("First resume failed: {:?}", e),
    }

    match co.resume::<()>(()) {
        Ok(_) => println!("Second resume: OK"),
        Err(e) => println!("Second resume failed: {:?}", e),
    }

    match co.resume::<()>(()) {
        Ok(_) => println!("Third resume: OK"),
        Err(e) => println!("Third resume failed: {:?}", e),
    }

    match co.resume::<()>(()) {
        Ok(_) => println!("Fourth resume: OK"),
        Err(e) => println!("Fourth resume failed: {:?}", e),
    }

    println!("Thread status: {:?}", co.status());

    Ok(())
}
