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
    commands_choice:fltk::misc::InputChoice,
    commands_input:fltk::input::Input,
}
#[derive(Clone)]
enum Message{
    DropdownCommand,
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
const TILE_SIZE:(i32,i32)=(WINDOW_SIZE.0,128);
use cube_combination_reader::*;
use fltk::prelude::WidgetExt;
use super::tui::TUI;
impl App{
    pub fn gui_loop(&mut self,file_opt:Option<String>){
        use std::collections::VecDeque;
        let (s,r)=app::channel::<Message>();
        self.parse_button=fltk::button::Button::default().with_label("ParseCommand");
        let (pb_p,pb_s)=(ScaleOffsetSize::new(0.75,0.5,0,0),ScaleOffsetSize::new(0.25,0.5,0,0));
        self.command_usage_frame=fltk::frame::Frame::default().with_label("Type commands on the bottom left or use the dropdown menu.");
        let (cuf_p,cuf_s)=(ScaleOffsetSize::new(0.0,0.0,0,0),ScaleOffsetSize::new(1.0,0.5,0,0));
        self.commands_choice=fltk::misc::InputChoice::default().with_label("Commands");
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
        self.tui.hm_command.remove(&"usage"); //Already shows a label of a command's usage
        let mut sort_cmds=self.tui.hm_command.iter().collect::<Box<_>>();
        sort_cmds.sort_unstable_by(|l,r|l.0.cmp(r.0));
        for (k,_) in sort_cmds.iter(){
            self.commands_choice.add(k);
        }
        self.commands_choice.emit(s.clone(),Message::DropdownCommand);
        self.parse_tile.add(&self.command_usage_frame);
        self.parse_tile.add(&self.parse_button);
        self.parse_tile.add(&self.commands_choice);
        self.parse_tile.add(&self.commands_input);
        self.window.add(&self.parse_tile);
        self.window.add(&self.output_box);
        self.window.show();
        let mut output_widgets:VecDeque<Box<dyn WidgetExt>>=VecDeque::new();
        let mut scroll_interpolate:i32=0;
        let mut do_scroll:bool=false;
        ///When adding/removing widgets.
        fn rearrange_widgets(ow:&mut VecDeque<Box<dyn WidgetExt>>,scroll_interpolate:&mut i32,do_scroll:&mut bool){
            let mut total_box_size:i32=0;
            for w in ow.iter_mut(){
                w.set_pos(0,total_box_size);
                total_box_size+=w.h();
            }
            *scroll_interpolate=total_box_size-WINDOW_SIZE.1+TILE_SIZE.1+12;
            *do_scroll=true;
        }
        let default_cmd=&(TUI::not_found_cmd as super::commands::Commands,"","");
        if let Some(file)=file_opt{
            let mut output=fltk::output::MultilineOutput::new(0,0,0,0,"");
            let command_unwrap_tup=self.tui.hm_command.get("load_from").expect("Wrong command.");
            if let Err(e)=command_unwrap_tup.0(&mut self.tui,&["",file.as_str()],""
                ,&mut super::IOWrapper::FltkOutput(&mut output)){
                    output.set_value(&format!("Error has occured: {e:?}: {e}\n"));
            }
            let value_ncount=output.value().chars().fold(0,|acc,ch|acc+if ch=='\n'{1}else{0});
            output.set_size(WINDOW_SIZE.0,value_ncount*(output.text_size()+6)+output.text_size());
            self.output_box.add(&output);
            output_widgets.push_back(Box::new(output));
            rearrange_widgets(&mut output_widgets,&mut scroll_interpolate, &mut do_scroll);
            self.output_box.redraw();
        }
        while self.app.wait(){
            if do_scroll{
                self.output_box.scroll_to(0,scroll_interpolate); //Fixing dumb bug that keeps scrolling in the wrong position.
                if scroll_interpolate>self.output_box.yposition(){ continue; }else{ do_scroll=false; }
            }
            if let Some(msg)=r.recv(){
                match msg{
                    Message::DropdownCommand=>{
                        if let Some(current_value)=self.commands_choice.value(){
                            if current_value.is_empty(){
                                self.command_usage_frame.set_label("Type commands on the bottom left or use the dropdown menu.");
                                continue
                            }
                            let Some(command_unwrap_tup)=self.tui.hm_command.get(current_value.as_str()) else{
                                let result=self.tui.hm_command.keys().filter(|&k|
                                    k.to_lowercase().contains(&current_value.as_str().to_lowercase()))
                                    .fold(String::new(),|res,s| res+s+". ");
                                self.command_usage_frame.set_label(format!("'{current_value}' is not a valid command. Possible matches: {result}").as_str());
                                continue
                            };
                            self.command_usage_frame.set_label(format!("{}\n{}",command_unwrap_tup.1,command_unwrap_tup.2).as_str());
                            match current_value.as_str(){
                                "load_from"|"save_to"|"write_to"=>{
                                    let mut file_dialog=fltk::dialog::NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseFile);
                                    file_dialog.show();
                                    if let Some(file_str)=file_dialog.filename().to_str(){
                                        self.commands_input.set_value(file_str);
                                    }
                                }
                                _=>{}
                            }
                            self.app.redraw();
                        }
                    }
                    Message::CommandParse=>{
                        if let Some(ch)=self.commands_choice.value(){
                            let cmd_str=format!("{ch} {}",self.commands_input.value());
                            let args:Box<_>=cmd_str.split_whitespace().collect();
                            if args.is_empty(){ continue }
                            let command_unwrap_tup=self.tui.hm_command.get(args[0]).unwrap_or(default_cmd);
                            let usage_str=command_unwrap_tup.1.to_string();
                            let mut output=fltk::output::MultilineOutput::new(0,0,0,0,"");
                            if let Err(e)=command_unwrap_tup.0(&mut self.tui,&args,&usage_str
                                ,&mut super::IOWrapper::FltkOutput(&mut output)){
                                output.set_value(&format!("Error has occured: {e:?}: {e}\n"));
                            }
                            let value_ncount=output.value().chars().fold(0,|acc,ch|acc+if ch=='\n'{1}else{0});
                            output.set_size(WINDOW_SIZE.0,value_ncount*(output.text_size()+6)+output.text_size());
                            self.output_box.add(&output);
                            output_widgets.push_back(Box::new(output));
                            rearrange_widgets(&mut output_widgets,&mut scroll_interpolate, &mut do_scroll);
                            self.tui.set_save_flag(args[0]);
                            if self.tui.is_program_done(){ break }
                        }
                        self.app.redraw();
                    }
                    Message::TileDrag=>{
                        self.app.redraw();
                    }
                }
            }
        }
    }
}