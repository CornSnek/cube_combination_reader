mod cube;
fn main(){
    let args=std::env::args().collect::<Box<_>>();
    let args_len@(2|3)=args.len() else{
        eprintln!("Usage: (program name) terminal|gui (optional file_name)");
        std::process::exit(1)
    };
    match args[1].as_str(){
        "terminal"=>{
            let mut tui:cube::tui::TUI=Default::default();
            tui.terminal_loop(if args_len==3{Some(args[2].clone())}else{None});
        }
        "gui"=>{
            let mut app:cube::fltk_ui::App=Default::default();
            app.gui_loop(if args_len==3{Some(args[2].clone())}else{None});
        }
        _=>{
            eprintln!("Usage: First argument needs to be 'terminal|gui'");
            std::process::exit(1)
        }
    }
}
