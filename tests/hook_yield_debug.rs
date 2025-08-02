use mlua::prelude::*;

#[test]
fn test_hook_yield_debug() -> mlua::Result<()> {
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

    co.set_hook(mlua::HookTriggers::EVERY_LINE, move |_lua, _debug| Ok(mlua::VmState::Yield))?;

    println!("Starting hook yield test...");
    
    match co.resume::<()>(()) {
        Ok(_) => println!("First resume: OK"),
        Err(e) => {
            println!("First resume failed: {:?}", e);
            return Err(e);
        }
    }
    
    match co.resume::<()>(()) {
        Ok(_) => println!("Second resume: OK"),
        Err(e) => {
            println!("Second resume failed: {:?}", e);
            return Err(e);
        }
    }
    
    match co.resume::<()>(()) {
        Ok(_) => println!("Third resume: OK"),
        Err(e) => {
            println!("Third resume failed: {:?}", e);
            return Err(e);
        }
    }
    
    match co.resume::<()>(()) {
        Ok(_) => println!("Fourth resume: OK"),
        Err(e) => {
            println!("Fourth resume failed: {:?}", e);
            return Err(e);
        }
    }
    
    println!("Thread status: {:?}", co.status());
    
    Ok(())
}
