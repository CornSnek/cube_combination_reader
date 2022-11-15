use fltk::{app, prelude::*, window::{Window, self}};
pub struct App{
    app:app::App,
    window:window::DoubleWindow,
    max_output_widgets:usize,
    tui:super::tui::TUI,
    parse_tile:fltk::group::Tile,
    output_box:fltk::group::Scroll,
    command_usage_frame:fltk::frame::Frame,
    parse_button:fltk::button::Button,
    commands_choice:fltk::menu::Choice,
    commands_input:fltk::input::Input,
}
#[derive(Clone)]
enum Message{
    DropdownCommand{syntax:String,usage:String},
    CommandParse,
    TileDrag,
}
impl Default for App{
    fn default()->Self {
        Self{
            app:app::App::default().with_scheme(app::Scheme::Gtk),
            window:Window::new(0,0,WINDOW_SIZE.0,WINDOW_SIZE.1,"Cube Combinations Reader"),
            max_output_widgets:10,
            tui:Default::default(),
            parse_tile:Default::default(),
            output_box:Default::default(),
            command_usage_frame:Default::default(),
            parse_button:Default::default(),
            commands_choice:Default::default(),
            commands_input:Default::default(),
        }
    }
}
const WINDOW_SIZE:(i32,i32)=(1024,768);
const TILE_SIZE:(i32,i32)=(WINDOW_SIZE.0,96);
use cube_combination_reader::*;
use fltk::prelude::WidgetExt;
use super::tui::TUI;
impl App{
    pub fn run(&mut self){
        use std::collections::VecDeque;
        let (s,r)=app::channel::<Message>();
        self.parse_button=fltk::button::Button::default().with_label("ParseCommand");
        let (pb_p,pb_s)=(ScaleOffsetSize::new(0.75,0.5,0,0),ScaleOffsetSize::new(0.25,0.5,0,0));
        self.command_usage_frame=fltk::frame::Frame::default();
        let (cuf_p,cuf_s)=(ScaleOffsetSize::new(0.0,0.0,0,0),ScaleOffsetSize::new(1.0,0.5,0,0));
        self.commands_choice=fltk::menu::Choice::default().with_label("Commands");
        let (cc_p,cc_s)=(ScaleOffsetSize::new(0.0,0.5,0,0),ScaleOffsetSize::new(0.25,0.5,0,0));
        self.commands_input=fltk::input::Input::default();
        let (ci_p,ci_s)=(ScaleOffsetSize::new(0.25,0.5,0,0),ScaleOffsetSize::new(0.5,0.5,0,0));
        self.parse_tile=fltk::group::Tile::new(0,0,TILE_SIZE.0,TILE_SIZE.1,"tile");
        self.output_box=fltk::group::Scroll::new(0,TILE_SIZE.1,TILE_SIZE.0,WINDOW_SIZE.1-TILE_SIZE.1,"");
        fn format_widget<T:WidgetExt,U:WidgetExt>(widget:&mut T,parent:&U,p:&ScaleOffsetSize,s:&ScaleOffsetSize){
            let (p_x,p_y,s_x,s_y)=(parent.x(),parent.y(),parent.w(),parent.h());
            widget.set_pos(p.x((p_x,p_y),(s_x,s_y)),p.y((p_x,p_y),(s_x,s_y)));
            widget.set_size(s.x((p_x,p_y),(s_x,s_y)),s.y((p_x,p_y),(s_x,s_y)));
        }
        format_widget(&mut self.parse_button,&self.parse_tile,&pb_p,&pb_s);
        format_widget(&mut self.command_usage_frame,&self.parse_tile,&cuf_p,&cuf_s);
        format_widget(&mut self.commands_choice,&self.parse_tile,&cc_p,&cc_s);
        format_widget(&mut self.commands_input,&self.parse_tile,&ci_p,&ci_s);
        self.parse_button.emit(s.clone(),Message::CommandParse);
        self.parse_tile.emit(s.clone(),Message::TileDrag);
        self.tui.hm_command.remove(&"exit"); //Not useful in gui.
        self.tui.hm_command.remove(&"usage"); //Already shows a label of a command's usage
        let mut sort_cmds=self.tui.hm_command.iter().collect::<Box<_>>();
        sort_cmds.sort_unstable_by(|l,r|l.0.cmp(r.0));
        for (k,tup) in sort_cmds.iter(){
            self.commands_choice.add_emit(k,
                fltk::enums::Shortcut::None,
                fltk::menu::MenuFlag::Normal,
                s.clone(),
                Message::DropdownCommand{syntax:tup.1.to_string(),usage:tup.2.to_string()}
            );
        }
        self.parse_tile.add(&self.command_usage_frame);
        self.parse_tile.add(&self.parse_button);
        self.parse_tile.add(&self.commands_choice);
        self.parse_tile.add(&self.commands_input);
        self.window.add(&self.parse_tile);
        self.window.add(&self.output_box);
        self.window.show();
        let mut output_widgets:VecDeque<Box<dyn WidgetExt>>=VecDeque::new();
        ///When adding/removing widgets.
        fn rearrange_widgets(ow:&mut VecDeque<Box<dyn WidgetExt>>){
            let mut total_text_size:i32=0;
            for w in ow.iter_mut(){
                w.set_pos(0,total_text_size);
                total_text_size+=w.h();
            }
        }
        while self.app.wait(){
            if let Some(msg)=r.recv(){
                match msg{
                    Message::DropdownCommand{syntax,usage}=>{
                        self.command_usage_frame.set_label(format!("{syntax}\n{usage}").as_str());
                        self.app.redraw();
                    }
                    Message::CommandParse=>{
                        if let Some(ch)=self.commands_choice.choice(){
                            let cmd_str=format!("{ch} {}",self.commands_input.value());
                            let args:Box<_>=cmd_str.split_whitespace().collect();
                            let default_cmd=&(TUI::not_found_cmd as super::commands::Commands,"","");
                            let command_unwrap_tup=self.tui.hm_command.get(args[0]).unwrap_or(default_cmd);
                            let usage_str=command_unwrap_tup.1.to_string();
                            let mut output=fltk::output::MultilineOutput::new(0,0,0,0,"");
                            if let Err(e)=command_unwrap_tup.0(&mut self.tui,&args,&usage_str
                                ,&mut super::IOWrapper::FltkOutput(&mut output)){
                                output.set_value(&format!("Error has occured: {e:?}: {e}"));
                            }
                            let value_ncount=output.value().chars().fold(0,|acc,ch|acc+if ch=='\n'{1}else{0});
                            output.set_size(WINDOW_SIZE.0,value_ncount*(output.text_size())); //TODO:Fix scollbar and scroll correctly
                            //output.total_text_size
                            self.output_box.add(&output);
                            output_widgets.push_back(Box::new(output));
                            rearrange_widgets(&mut output_widgets);
                            self.output_box.scroll_to(0,1000);
                            self.tui.set_save_flag(args[0]);
                        }
                        self.app.redraw();
                    }
                    Message::TileDrag=>{
                        self.app.redraw();
                    }
                }
            }
        }
        //self.tui.terminal_loop();
    }
}