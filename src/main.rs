mod cube;
fn main()->std::io::Result<()>{
    let mut tui_obj=cube::TUI::new();
    tui_obj.program_loop()?;
    Ok(())
}
