
use fltk::{app, prelude::*, window::{Window, self}};
pub struct App{
    app:app::App,
    window:window::DoubleWindow,
    tui:super::tui::TUI,
    frame:fltk::frame::Frame,
    button:fltk::button::Button,
    commands_choice:fltk::menu::Choice,
}
#[derive(Clone)]
enum Message{
    CommandParse
}
impl Default for App{
    fn default()->Self {
        Self{
            app:app::App::default(),
            window:Window::new(0,0,1024,768,"Cube Combinations Reader"),
            tui:Default::default(),
            frame:Default::default(),
            button:Default::default(),
            commands_choice:Default::default(),
        }
    }
}
impl App{
    pub fn run(&mut self){
        self.button=fltk::button::Button::new(100,100,100,100,"Parse Command");
        self.frame=fltk::frame::Frame::default().with_size(200,200).center_of(&self.window);
        self.commands_choice=fltk::menu::Choice::default();
        let (s,r)=app::channel::<Message>();
        self.button.emit(s,Message::CommandParse);
        self.window.add(&self.frame);
        self.window.add(&self.button);
        self.window.show();
        while self.app.wait(){
            if let Some(msg)=r.recv(){
                match msg{
                    Message::CommandParse=>{
                        self.frame.set_label("Parsed Command");
                    }
                }
            }
        }
        self.tui.terminal_loop();
    }
}