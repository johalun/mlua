use mlua::prelude::*;
use mlua::{HookTriggers, VmState};

#[test]
fn test_only_custom_hooks() -> mlua::Result<()> {
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

    // Set only custom hooks, not EVERY_LINE
    co.set_hook(HookTriggers { on_resume: true, on_yield: true, ..Default::default() }, 
                move |_lua, _debug| Ok(VmState::Continue))?;

    println!("Starting resumes...");
    match co.resume::<()>(()) {
        Ok(_) => println!("First resume: OK"),
        Err(e) => println!("First resume failed: {:?}", e),
    }

    println!("Thread status: {:?}", co.status());

    Ok(())
}
