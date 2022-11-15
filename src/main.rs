mod cube;
fn main(){
    let args=std::env::args().collect::<Box<_>>();
    if args.len()==1{
        eprintln!("Usage: To start the program, type 'terminal' or 'gui' after the program.");
        std::process::exit(1)
    }
    match args[1].as_str(){
        "terminal"=>{
            let mut tui:cube::tui::TUI=Default::default();
            tui.terminal_loop();
        }
        "gui"=>{
            let mut app:cube::fltk_ui::App=Default::default();
            app.run();
        }
        _=>{
            eprintln!("Usage: First argument needs to be 'terminal' or 'gui'");
            std::process::exit(1)
        }
    }
}
