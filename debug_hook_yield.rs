use mlua::prelude::*;

fn main() -> Result<()> {
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

    match co.resume::<()>(()) {
        Ok(_) => println!("First resume: OK"),
        Err(e) => println!("First resume failed: {:?}", e),
    }

    match co.resume::<()>(()) {
        Ok(_) => println!("Second resume: OK"),
        Err(e) => println!("Second resume failed: {:?}", e),
    }

    println!("Thread status: {:?}", co.status());

    Ok(())
}
