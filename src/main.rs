mod cube;
fn main(){
    let args=std::env::args().collect::<Box<_>>();
    let args_len@(2|3)=args.len() else{
        eprintln!("Usage: (program name) terminal|gui (optional file_name)?");
        std::process::exit(1)
    };
    match args[1].as_str(){
        "terminal"=>{
            let mut tui:cube::tui::TUI=Default::default();
            tui.terminal_loop(if args_len==3{Some(args[2].clone())}else{None});
        }
        "gui"=>{
            use fltk::{prelude::*,app::{App,Scheme},window::Window};
            use cube::fltk_ui::WINDOW_SIZE;
            let app=App::default().with_scheme(Scheme::Gtk);
            let window=Window::new(0,0,WINDOW_SIZE.0,WINDOW_SIZE.1,"Cube Combinations Reader");
            let mut fltk_app:cube::fltk_ui::FltkApp=cube::fltk_ui::FltkApp::new();
            fltk_app.gui_loop(app,window,if args_len==3{Some(args[2].clone())}else{None});
        }
        _=>{
            eprintln!("Usage: First argument needs to be 'terminal|gui'");
            std::process::exit(1)
        }
    }
}
