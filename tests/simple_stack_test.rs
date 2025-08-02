use mlua::prelude::*;

#[test]
fn test_simple_stack_corruption() -> mlua::Result<()> {
    let lua = Lua::new();
    
    let thread = lua.create_thread(
        lua.load(r#"
            local x = 5
            return x
        "#).into_function()?,
    )?;
    
    // Set a simple hook that doesn't yield
    thread.set_hook(
        mlua::HookTriggers {
            on_resume: true,
            ..Default::default()
        },
        |_lua, _debug| {
            println!("Resume hook fired");
            Ok(mlua::VmState::Continue)
        },
    )?;
    
    let result = thread.resume::<i32>(())?;
    println!("Result: {}", result);
    assert_eq!(result, 5);
    
    Ok(())
}
