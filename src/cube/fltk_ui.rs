use fltk::{app, prelude::*, window::{Window, self}};
pub struct App{
    app:app::App,
    window:window::DoubleWindow,
    max_output_widgets:usize,
    tui:super::tui::TUI,
    parse_tile:fltk::group::Tile,
    output_box:fltk::group::Scroll,
    command_usage_label:fltk::frame::Frame,
    parse_button:fltk::button::Button,
    commands_choice:fltk::misc::InputChoice,
    commands_input:fltk::input::Input,
    cube_data:fltk::misc::InputChoice,
    cube_data_label:fltk::frame::Frame,
    cube_data_add:fltk::button::Button,
}
#[derive(Clone)]
enum Message{
    DropdownCommand,
    CommandParse,
    CubeFilter,
    CubeAdd,
}
impl Default for App{
    fn default()->Self {
        Self{
            app:app::App::default().with_scheme(app::Scheme::Gtk),
            window:Window::new(0,0,WINDOW_SIZE.0,WINDOW_SIZE.1,"Cube Combinations Reader"),
            max_output_widgets:20,
            tui:Default::default(),
            parse_tile:Default::default(),
            output_box:Default::default(),
            command_usage_label:Default::default(),
            parse_button:Default::default(),
            commands_choice:Default::default(),
            commands_input:Default::default(),
            cube_data:Default::default(),
            cube_data_label:Default::default(),
            cube_data_add:Default::default(),
        }
    }
}
const WINDOW_SIZE:(i32,i32)=(1024,768);
const TILE_SIZE:(i32,i32)=(WINDOW_SIZE.0,128);
use cube_combination_reader::*;
use fltk::prelude::WidgetExt;
use super::tui::TUI;
///Abstraction for output widgets in the gui for different commands.
pub mod app_utils{
    pub enum OutputWidget{ //Explicit because self.output_box.remove cannot use Box<dyn WidgetExt>.
        MLO(fltk::output::MultilineOutput),
        ///Will add more output widgets later.
        #[allow(dead_code)] TempDummy
    }
    pub struct OutputContainer{
        pub cmd:fltk::frame::Frame,
        pub ow:Option<OutputWidget>,
    }
    impl OutputContainer{
        pub fn new(cmd:&String)->Self{
            use fltk::prelude::WidgetExt;
            let mut s=Self{..Default::default()};
            s.cmd.set_label(&("Command used: '".to_string()+&cmd+"'"));
            s
        }
        pub fn fltk_add(&mut self,p:&mut fltk::group::Scroll){
            use fltk::prelude::*;
            match self.ow{
                Some(OutputWidget::MLO(ref mut mlo))=>{
                    let str_v=mlo.value();
                    let value_ncount=str_v.chars().fold(0,|acc,ch|acc+if ch=='\n'{1}else{0});
                    let max_line_count=str_v.split('\n').map(|str|{ str.len() }).max().unwrap() as i32;
                    self.cmd.set_size(super::WINDOW_SIZE.0,20);
                    mlo.set_size((max_line_count*(mlo.text_size() as f32*0.6) as i32).max(super::WINDOW_SIZE.0),value_ncount*(mlo.text_size()+6)+mlo.text_size());
                    p.add(mlo);
                }
                _=>{}
            }
            p.add(&self.cmd);
        }
        pub fn fltk_remove(&mut self,p:&mut fltk::group::Scroll){
            use fltk::prelude::*;
            match self.ow{
                Some(OutputWidget::MLO(ref mlo))=>{
                    p.remove(mlo);
                }
                _=>{}
            }
            p.remove(&self.cmd);
        }
    }
    impl Default for OutputContainer{
        fn default()->Self{
            Self{cmd:Default::default(),ow:Default::default()}
        }
    }
}
impl App{
    pub fn gui_loop(&mut self,file_opt:Option<String>){
        use std::collections::VecDeque;
        let (s,r)=app::channel::<Message>();
        self.command_usage_label.set_label("Type commands on the bottom left or use the dropdown menu.");
        let (cuf_p,cuf_s)=(ScaleOffsetSize::new(0.0,0.0,0,0),ScaleOffsetSize::new(1.0,0.5,0,0));
        self.parse_button.set_label("ParseCommand");
        let (pb_p,pb_s)=(ScaleOffsetSize::new(0.75,0.5,0,0),ScaleOffsetSize::new(0.25,0.25,0,0));
        self.commands_choice.set_label("Commands");
        let (cc_p,cc_s)=(ScaleOffsetSize::new(0.0,0.5,0,0),ScaleOffsetSize::new(0.25,0.25,0,0));
        //self.commands_input=fltk::input::Input::default();
        let (ci_p,ci_s)=(ScaleOffsetSize::new(0.25,0.5,0,0),ScaleOffsetSize::new(0.5,0.25,0,0));
        //self.cube_data=fltk::misc::InputChoice::default();
        let (cd_p,cd_s)=(ScaleOffsetSize::new(0.0,0.75,0,0),ScaleOffsetSize::new(0.25,0.25,0,0));
        self.cube_data_label.set_label("Cube filter: 0 total cubes. Found 0.");
        let (cdl_p,cdl_s)=(ScaleOffsetSize::new(0.5,0.75,0,0),ScaleOffsetSize::new(0.5,0.25,0,0));
        self.cube_data_add.set_label("Add cube to arguments");
        let (cda_p,cda_s)=(ScaleOffsetSize::new(0.25,0.75,0,0),ScaleOffsetSize::new(0.25,0.25,0,0));
        self.parse_tile=fltk::group::Tile::new(0,0,TILE_SIZE.0,TILE_SIZE.1,"tile");
        self.output_box=fltk::group::Scroll::new(0,TILE_SIZE.1,TILE_SIZE.0,WINDOW_SIZE.1-TILE_SIZE.1,"");
        fn format_widget<T:WidgetExt,U:WidgetExt>(widget:&mut T,parent:&U,p:&ScaleOffsetSize,s:&ScaleOffsetSize){
            let (p_x,p_y,s_x,s_y)=(parent.x(),parent.y(),parent.w(),parent.h());
            widget.set_pos(p.x((p_x,p_y),(s_x,s_y)),p.y((p_x,p_y),(s_x,s_y)));
            widget.set_size(s.x((p_x,p_y),(s_x,s_y)),s.y((p_x,p_y),(s_x,s_y)));
        }
        format_widget(&mut self.parse_button,&self.parse_tile,&pb_p,&pb_s);
        format_widget(&mut self.command_usage_label,&self.parse_tile,&cuf_p,&cuf_s);
        format_widget(&mut self.commands_choice,&self.parse_tile,&cc_p,&cc_s);
        format_widget(&mut self.commands_input,&self.parse_tile,&ci_p,&ci_s);
        format_widget(&mut self.cube_data,&self.parse_tile,&cd_p,&cd_s);
        format_widget(&mut self.cube_data_label,&self.parse_tile,&cdl_p,&cdl_s);
        format_widget(&mut self.cube_data_add,&self.parse_tile,&cda_p,&cda_s);
        self.parse_button.emit(s.clone(),Message::CommandParse);
        self.tui.hm_command.remove("find");
        let mut sort_cmds=self.tui.hm_command.iter().collect::<Box<_>>();
        sort_cmds.sort_unstable_by(|l,r|l.0.cmp(r.0));
        for (k,_) in sort_cmds.iter(){
            self.commands_choice.add(k);
        }
        self.commands_choice.emit(s.clone(),Message::DropdownCommand);
        self.cube_data.emit(s.clone(),Message::CubeFilter);
        self.cube_data_add.emit(s.clone(),Message::CubeAdd);
        self.parse_tile.add(&self.command_usage_label);
        self.parse_tile.add(&self.parse_button);
        self.parse_tile.add(&self.commands_choice);
        self.parse_tile.add(&self.commands_input);
        self.parse_tile.add(&self.cube_data);
        self.parse_tile.add(&self.cube_data_label);
        self.parse_tile.add(&self.cube_data_add);
        self.window.add(&self.parse_tile);
        self.window.add(&self.output_box);
        self.window.show();
        use app_utils::*;
        let mut output_widgets:VecDeque<OutputContainer>=VecDeque::new();
        let mut scroll_interpolate:i32=0;
        let mut do_scroll:bool=false;
        ///When adding/removing widgets.
        fn rearrange_widgets(ow:&mut VecDeque<OutputContainer>,scroll_interpolate:&mut i32,do_scroll:&mut bool){
            let mut total_box_size:i32=0;
            for w in ow.iter_mut(){
                w.cmd.set_pos(0,total_box_size);
                total_box_size+=w.cmd.h();
                match w.ow{
                    Some(OutputWidget::MLO(ref mut mlo))=>{
                        mlo.set_pos(0,total_box_size);
                        total_box_size+=mlo.h();
                    }
                    _=>{}
                }
            }
            *scroll_interpolate=total_box_size-WINDOW_SIZE.1+TILE_SIZE.1+12;
            *do_scroll=true;
        }
        fn do_cube_filter_search(tui:&TUI,cube_data:&mut fltk::misc::InputChoice,cube_data_label:&mut fltk::frame::Frame){
            cube_data.clear();
            let filter_str=cube_data.value();
            cube_data_label.set_label(format!("Cube filter: {} total cubes. Found {}."
                ,tui.cube_count(None),tui.cube_count(filter_str.clone())).as_str());
            for cube in tui.cube_keys(filter_str).into_iter(){
                cube_data.add(cube.as_str());
            }
        }
        let default_cmd=&(TUI::not_found_cmd as super::commands::Commands,"","");
        if let Some(file)=file_opt{
            let command_unwrap_tup=self.tui.hm_command.get("load_from").expect("Wrong command.");
            let mut oc=OutputContainer::new(&"load_from".to_string());
            if let Err(e)=command_unwrap_tup.0(&mut self.tui,&["",file.as_str()],""
                ,&mut super::IOWrapper::FltkOutput(&mut oc)){
                    if let Some(OutputWidget::MLO(ref mut mlo))=oc.ow{
                        mlo.set_value(&format!("Error has occured: {e:?}: {e}\n"));
                    }if let None=oc.ow{
                        let mut mlo=fltk::output::MultilineOutput::default();
                        mlo.set_value(&format!("Error has occured: {e:?}: {e}\n"));
                        oc.ow=Some(OutputWidget::MLO(mlo));
                    }else{
                        todo!("Remove other widgets.")
                    }
            }
            oc.fltk_add(&mut self.output_box);
            output_widgets.push_back(oc);
            rearrange_widgets(&mut output_widgets,&mut scroll_interpolate,&mut do_scroll);
            self.output_box.redraw();
        }
        while self.app.wait(){
            self.app.redraw();
            if do_scroll{
                self.output_box.scroll_to(0,scroll_interpolate); //Fixing dumb bug that keeps scrolling in the wrong position.
                if scroll_interpolate>self.output_box.yposition(){ continue; }else{ do_scroll=false; }
            }
            if let Some(msg)=r.recv(){
                match msg{
                    Message::DropdownCommand=>{
                        if let Some(current_value)=self.commands_choice.value(){
                            if current_value.is_empty(){
                                self.command_usage_label.set_label("Type commands on the bottom left or use the dropdown menu.");
                                continue
                            }
                            let Some(command_unwrap_tup)=self.tui.hm_command.get(current_value.as_str()) else{
                                let result=self.tui.hm_command.keys().filter(|&k|
                                    k.to_lowercase().contains(&current_value.as_str().to_lowercase()))
                                    .fold(String::new(),|res,s| res+s+". ");
                                self.command_usage_label.set_label(format!("'{current_value}' is not a valid command. Possible matches: {result}").as_str());
                                continue
                            };
                            self.command_usage_label.set_label(format!("{}\n{}",command_unwrap_tup.1,command_unwrap_tup.2).as_str());
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
                        }
                    }
                    Message::CommandParse=>{
                        if let Some(ch)=self.commands_choice.value(){
                            let cmd_str=format!("{ch} {}",self.commands_input.value());
                            let args:Box<_>=cmd_str.split_whitespace().collect();
                            if args.is_empty(){ continue }
                            let command_unwrap_tup=self.tui.hm_command.get(args[0]).unwrap_or(default_cmd);
                            let usage_str=command_unwrap_tup.1.to_string();
                            let mut oc=OutputContainer::new(&cmd_str);
                            if let Err(e)=command_unwrap_tup.0(&mut self.tui,&args,&usage_str
                                ,&mut super::IOWrapper::FltkOutput(&mut oc)){
                                    if let Some(OutputWidget::MLO(ref mut mlo))=oc.ow{
                                        mlo.set_value(&format!("Error has occured: {e:?}: {e}\n"));
                                    }if let None=oc.ow{
                                        let mut mlo=fltk::output::MultilineOutput::default();
                                        mlo.set_value(&format!("Error has occured: {e:?}: {e}\n"));
                                        oc.ow=Some(OutputWidget::MLO(mlo));
                                    }else{
                                        todo!("Remove other widgets.")
                                    }
                            }
                            oc.fltk_add(&mut self.output_box);
                            output_widgets.push_back(oc);
                            if output_widgets.len()>self.max_output_widgets{
                                output_widgets.pop_front().unwrap().fltk_remove(&mut self.output_box);
                            }
                            rearrange_widgets(&mut output_widgets,&mut scroll_interpolate,&mut do_scroll);
                            self.tui.set_save_flag(args[0]);
                            if self.tui.is_program_done(){ break }
                        }
                        self.cube_data.clear();
                        do_cube_filter_search(&self.tui,&mut self.cube_data,&mut self.cube_data_label);
                    }
                    Message::CubeFilter=>{
                        if !self.cube_data.changed(){ //Changed via typing.
                            do_cube_filter_search(&mut self.tui,&mut self.cube_data,&mut self.cube_data_label);
                        }
                    }
                    Message::CubeAdd=>{
                        let old_str=self.commands_input.value();
                        let Some(value_str)=self.cube_data.value() else{
                            continue
                        };
                        self.commands_input.set_value(&(old_str+" "+&value_str));
                    }
                }
            }
        }
    }
}