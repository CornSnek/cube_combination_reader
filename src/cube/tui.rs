pub struct TUI{
    cdll:super::CubeDLL,
    done_program:bool,
    pub(super) hm_command:super::commands::CommandHashMap<'static>,
    has_saved:bool
}
pub fn get_commands_hashmap()->super::commands::CommandHashMap<'static>{
    let mut cmd_hm=super::commands::CommandHashMap::new();
    cmd_hm.insert("add",(TUI::add_cmd,"<add ((cube_name) (Tier))+>","Adds cube with name and tier"));
    cmd_hm.insert("remove",(TUI::rm_cmd,"<destroy|drop|remove (cube_name)+>","Destroys a cube and all its links with other cubes"));
    cmd_hm.insert("drop",(TUI::rm_cmd,"<destroy|drop|remove (cube_name)+>","Destroys a cube and all its links with other cubes"));
    cmd_hm.insert("destroy",(TUI::rm_cmd,"<destroy|drop|remove (cube_name)+>","Destroys a cube and all its links with other cubes"));
    cmd_hm.insert("rename",(TUI::rename_cmd,"<rename (cube) to (new_name)>","Renames a cube and all its links with will be changed to that name"));
    cmd_hm.insert("read",(TUI::read_cmd,"<read (cube_name)+>","Reads a cube's properties and also reads cubes with associated fusions"));
    cmd_hm.insert("read_all",(TUI::read_all_cmd,"<read_all>","Reads all cubes' properties in the program"));
    cmd_hm.insert("link",(TUI::link_cmd,"<fuse|link (cube_name) with (cube_1) (cube_2)?>","Links 1 or 2 cubes together for a cube_name")); 
    cmd_hm.insert("fuse",(TUI::link_cmd,"<fuse|link (cube_name) with (cube_1) (cube_2)?>","Links 1 or 2 cubes together for a cube_name"));
    cmd_hm.insert("merge",(TUI::merge_cmd,"<merge (cube_a) (cube_b) in (cube_name)>","Merge links of single cubes using link commands"));
    cmd_hm.insert("unlink",(TUI::unlink_cmd,"<unfuse|unlink (cube_1) (cube_2)? in (cube_name)>","Unlinks cube fusions for the specified cube_name"));
    cmd_hm.insert("unfuse",(TUI::unlink_cmd,"<unfuse|unlink (cube_1) (cube_2)? in (cube_name)>","Unlinks cube fusions for the specified cube_name"));
    cmd_hm.insert("unlink_all",(TUI::unlink_all_cmd,"<unfuse_all|unlink_all (cube_name)+>","Unlinks all fusions and associations of conversions for a cube"));
    cmd_hm.insert("unfuse_all",(TUI::unlink_all_cmd,"<unfuse_all|unlink_all (cube_name)+>","Unlinks all fusions and associations of conversions for a cube"));
    cmd_hm.insert("exit",(TUI::exit_cmd,"<exit>","Exits the program. Will ask to save unsaved data if any."));
    cmd_hm.insert("save_to",(TUI::save_to_cmd,"<save_to (file_name)>","Saves data to a text file"));
    cmd_hm.insert("write_to",(TUI::save_to_cmd,"<save_to (file_name)>","Saves data to a text file"));
    cmd_hm.insert("load_from",(TUI::load_from_cmd,"<load_from (file_name)>","Loads data from a text file"));
    cmd_hm.insert("remove_all",(TUI::rem_all_cmd,"<destroy_all|drop_all|remove_all>","Clears all cube data in the program"));
    cmd_hm.insert("drop_all",(TUI::rem_all_cmd,"<destroy_all|drop_all|remove_all>","Clears all cube data in the program"));
    cmd_hm.insert("destroy_all",(TUI::rem_all_cmd,"<destroy_all|drop_all|remove_all>","Clears all cube data in the program"));
    cmd_hm.insert("change_tier",(TUI::change_tier_cmd,"<change_tier (cube_name) (this_tier)>","Changes a tier for a specified cube"));
    cmd_hm.insert("get_fusions",(TUI::get_fusions_cmd,"<get_fusions (cube_name)>","Gets all cube fusions for the specified cube. Similar to read, but prints/enumerates cube fusions instead."));
    cmd_hm.insert("build_tree",(TUI::build_tree_cmd,"<build_tree (cube_name)>","Gets all associated cube fusions to make this cube."));
    cmd_hm.insert("find",(TUI::find_cmd,"<find|search starts_with|contains (partial cube_name)>","Finds all associated cube names within the program (case-insensitive)"));
    cmd_hm.insert("search",(TUI::find_cmd,"<find|search starts_with|contains (partial cube_name)>","Finds all associated cube names within the program (case-insensitive)"));
    cmd_hm.insert("todo",(TUI::todo_cmd,"<todo>","Finds tiers that have not been set yet (-1), finds single fusions not set yet to double, and finds ? fusions."));
    cmd_hm.insert("usage",(TUI::usage_cmd,"<usage>","Sees all commands used within the program"));
    cmd_hm
}
impl TUI{ ///For the commands in get_commands_hashmap(), set the appropriate save flags when quitting the program.
    pub fn set_save_flag(&mut self,arg:&str){
        match arg{
            "destroy_all"|"drop_all"|"remove_all"|"save_to"|"write_to"|"load_from"| //These 6 implicitly do self.has_saved if y is entered.
            "read"|"read_all"|"get_fusions"|"build_tree"| //These commands and below just reads all the data or do nothing to the data.
            "find"|"search"|"usage"|"todo"|"exit"=>(),
            _=>self.has_saved=false,
        }
    }
}
macro_rules! do_print_error{
    ($($arg:tt)*)=>{ eprint!("\x1b[1;33m"); eprint!($($arg)*); eprintln!("\x1b[0m"); }
}
pub(crate) use do_print_error;
use std::collections::HashSet;
use std::io::Read;
use super::CubeStruct;
use super::error::CSError;
use super::error::CSResult;
use super::IOWrapper;
use rustyline::Editor;
use rustyline::config::Configurer;
///Asserting characters are UTF-8 valid and can't be null
fn check_valid_cube_name(arg:&str)->CSResult<()>{
    if arg=="?"{ return Ok(()) }
    for b in arg.as_bytes(){
        match b{
            b if b.is_ascii_alphanumeric() => continue,
            b'('|b')' => continue,
            _ => return Err(CSError::NameSyntax(arg.to_string()))
        }
    }
    Ok(())
}
impl Default for TUI{
    fn default()->Self{
        Self{cdll:Default::default(),
            done_program:false,
            hm_command:get_commands_hashmap(),
            has_saved:true
        }
    }
}
///All functions ending in _cmd is for a hashmap.
impl TUI{
    pub fn is_program_done(&self)->bool{
        self.done_program
    }
    pub fn terminal_loop(&mut self,file_opt:Option<String>){
        if let Some(file)=file_opt{
            let command_unwrap_tup=self.hm_command.get("load_from").expect("Wrong command.");
            if let Err(e)=command_unwrap_tup.0(self,&["",file.as_str()],""
                ,&mut IOWrapper::Stdio(&mut std::io::stdout(),&mut std::io::stdin())){
                do_print_error!("Error has occured: {e:?}: {e}");
            }
        }
        let mut rl=Editor::<()>::new().expect("Unable to setup program loop.");
        rl.set_max_history_size(100);
        if rl.load_history("cmd_history.txt").is_err(){ println!("Commands not saved yet.") }
        while !self.done_program{
            println!("\nType \"usage\" for commands. Type \"exit\" to exit the program");
            let readline=rl.readline("> ");
            use rustyline::error::ReadlineError;
            match readline{
                Ok(line)=>{
                    rl.add_history_entry(line.as_str());
                    let args:Box<_>=line.split_whitespace().collect();
                    let default_cmd=&(Self::not_found_cmd as super::commands::Commands,"","");
                    if args.is_empty(){ continue }
                    let command_unwrap_tup=self.hm_command.get(args[0]).unwrap_or(default_cmd);
                    if let Err(e)=command_unwrap_tup.0(self,&args,command_unwrap_tup.1,
                        &mut IOWrapper::Stdio(&mut std::io::stdout(),&mut std::io::stdin())){
                        do_print_error!("Error has occured: {e:?}: {e}");
                    }
                    self.set_save_flag(args[0]);
                },
                Err(ReadlineError::Io(e))=>{
                    do_print_error!("Error has occured: {e:?}: {e}");
                },
                Err(ReadlineError::Interrupted|ReadlineError::Eof)=>{
                    do_print_error!("Program has ended early.");
                    break
                }
                _=>{ 
                    do_print_error!("Unknown error in program. Exiting early.");
                    break
                }
            }
        }
        if rl.save_history("cmd_history.txt").is_err(){ do_print_error!("Couldn't save commands."); }
        if !self.has_saved{
            println!(concat!("Saving any unsaved data to ",SWF!(Temp),"."));
            if let Err(e)=self.save_temp(){
                do_print_error!("Saving error has occured: {e:?}: {e}");
            }
        }
    }
    fn add_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()%2==0||args.len()==1{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        for chunk in args[1..].chunks(2){
            check_valid_cube_name(chunk[0])?;
            if chunk[0]=="?"{ return Err(CSError::InvalidArguments("? cannot be added other than using it for linking".to_string())) }
            let is_tier_num=chunk[1].parse::<i32>();
            match is_tier_num{
                Ok(tier_num)=>{ self.cdll.add(CubeStruct::new(chunk[0].to_string(),tier_num))?; }
                Err(e)=> return ErrToCSErr!(e)
            }
            w.write_output_nl(format!("Cube \"{}\" added",chunk[0]))?;
        }
        Ok(())
    }
    fn rm_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()==1{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        for cube_str in &args[1..]{
            if cube_str==&"?"{ return Err(CSError::InvalidArguments("? cannot be destroyed. Use unlink command to remove ? links instead.".to_string())) }
            check_valid_cube_name(cube_str)?;
            self.cdll.point_to(cube_str.to_string())?;
            self.cdll.destroy_at_p(w)?;
            w.write_output_nl(format!("Cube \"{cube_str}\" and all of its links have been destroyed."))?;
        }
        Ok(())
    }
    fn rename_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()!=4||args[2]!="to"{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        if args[1]=="?"||args[3]=="?"{
            return Err(CSError::InvalidArguments("? cannot be renamed or be used as a name".to_string()))
        }
        check_valid_cube_name(args[1])?;
        check_valid_cube_name(args[3])?;
        self.cdll.point_to(args[1].to_string())?;
        self.cdll.rename_at_p(args[3].to_string(),w)?;
        w.write_output_nl(format!("Cube \"{}\" successfully renamed to \"{}\"",args[1],args[3]))?;
        Ok(())
    }
    fn read_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()==1{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        for arg in &args[1..]{
            check_valid_cube_name(arg)?;
            self.cdll.point_to(arg.to_string())?;
            self.cdll.get_info_p(w)?;
        }
        Ok(())
    }
    fn read_all_cmd(&mut self,_:&[&str],_:&str,w:&mut IOWrapper)->CSResult<()>{
        self.cdll.get_info_cube_paths(w)?;
        Ok(())
    }
    fn link_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        let n@(4|5)=args.len() else{
            return Err(CSError::InvalidArguments(usage.to_string()))
        };
        if args[2]!="with"{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        check_valid_cube_name(args[1])?;
        self.cdll.point_to(args[1].to_string())?;
        check_valid_cube_name(args[3])?;
        if args[3]=="?"{ return Err(CSError::InvalidArguments("? cannot be used to fuse with another cube".to_string())) }
        if n==4{
            self.cdll.link_at_p_fb_single(args[3].to_string(),w)?;
            w.write_output_nl(format!("Successfully linked cube \"{}\" with \"{}\"",args[1],args[3]))?;
        }else{
            check_valid_cube_name(args[4])?;
            if args[4]=="?"{ return Err(CSError::InvalidArguments("? cannot be used to fuse with another cube".to_string())) }
            self.cdll.link_at_p_fb_pair(args[3].to_string(),args[4].to_string(),w)?;
            w.write_output_nl(format!("Successfully linked cube \"{}\" with \"{}\" and \"{}\"",args[1],args[3],args[4]))?;
        }
        Ok(())
    }
    fn merge_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()!=5||args[3]!="in"{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        check_valid_cube_name(args[1])?;
        if args[1]=="?"{ return Err(CSError::InvalidArguments("? cannot be used to merge".to_string())) }
        check_valid_cube_name(args[2])?;
        if args[2]=="?"{ return Err(CSError::InvalidArguments("? cannot be used to merge".to_string())) }
        check_valid_cube_name(args[4])?;
        self.cdll.point_to(args[4].to_string())?;
        self.cdll.merge_keys_at_p(args[1].to_string(),args[2].to_string(),w)?;
        w.write_output_nl(format!("Successfully merged keys \"{}\" with \"{}\" for \"{}\"",args[1],args[2],args[4]))?;
        Ok(())
    }
    fn unlink_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        let n@(4|5)=args.len() else{
            return Err(CSError::InvalidArguments(usage.to_string()))
        };
        if args[n-2]!="in"{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        check_valid_cube_name(args[n-1])?;
        self.cdll.point_to(args[n-1].to_string())?;
        check_valid_cube_name(args[1])?;
        if args[1]=="?"{ return Err(CSError::InvalidArguments("? cannot be unlinked with another cube".to_string())) }
        if n==5 {
            check_valid_cube_name(args[2])?;
            if args[2]=="?"{ return Err(CSError::InvalidArguments("? cannot be unlinked with another cube".to_string())) }
        }
        self.cdll.unlink_at_p_fb_keys(args[1].to_string(),if n==5{ Some(args[2].to_string()) }else{ None },w)?;
        w.write_output_nl(format!("Successfully removed for cube \"{}\"",args[n-1]))?;
        Ok(())
    }
    fn unlink_all_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()==1{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        for arg in &args[1..]{
            check_valid_cube_name(arg)?;
            self.cdll.point_to(arg.to_string())?;
            self.cdll.unlink_at_p_fb()?;
            w.write_output_nl(format!("Successfully unlinked cube \"{arg}\"'s fused_by properties."))?;
        }
        Ok(())
    }
    fn change_tier_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()!=3{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        check_valid_cube_name(args[1])?;
        self.cdll.point_to(args[1].to_string())?;
        if args[1]=="?"{ return Err(CSError::InvalidArguments("? does not have a tier".to_string())) }
        let tier={ match args[2].parse::<i32>(){ Ok(tier)=>{ tier } Err(e)=> return ErrToCSErr!(e) } };
        self.cdll.change_tier_at_p(tier,w)?;
        Ok(())
    }
    fn rem_all_cmd(&mut self,_:&[&str],_:&str,w:&mut IOWrapper)->CSResult<()>{
        if self.yn_loop("All cube data will be erased without saving.".to_string(),w)?{
            self.cdll.remove_all_cubes();
            self.has_saved=true;
            w.write_output_nl("All cubes in the program have been removed.".to_string())?;
        }
        Ok(())
    }
    fn get_fusions_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()!=2{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        check_valid_cube_name(args[1])?;
        self.cdll.point_to(args[1].to_string())?;
        self.cdll.get_fusions_at_p(w)?;
        Ok(())
    }
    pub(super) fn build_tree_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()!=2{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        check_valid_cube_name(args[1])?;
        self.cdll.point_to(args[1].to_string())?;
        self.cdll.build_tree_at_p(w,args[0])?;
        Ok(())
    }
    fn find_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()!=3{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        check_valid_cube_name(args[2])?;
        let result=match args[1]{
            "starts_with"=>{ //to_lowercase to be case-insensitive
                w.write_output_nl(format!("Finding cube names starting with '{}'",args[2]))?;
                let result=self.cdll.hashmap.keys().filter(|&k|
                    k.to_lowercase().starts_with(&args[2].to_lowercase()))
                .enumerate().fold(String::new(),|res,t|
                    res+&((t.0+1).to_string())+": "+t.1+"\n"
                );
                if !result.is_empty(){
                    result[..=result.len()-2].to_string() //-2 to exclude trailing \n and 0-indexing
                }else{"(None found)".to_string()}
            }
            "contains"=>{
                w.write_output_nl(format!("Finding cube names containing substring '{}'",args[2]))?;
                let result=self.cdll.hashmap.keys().filter(|&k|
                    k.to_lowercase().contains(&args[2].to_lowercase()))
                .enumerate().fold(String::new(),|res,t|
                    res+&(t.0+1).to_string()+": "+t.1+"\n"
                );
                if !result.is_empty(){
                    result[..=result.len()-2].to_string()
                }else{"(None found)".to_string()}
            }
            _=>return Err(CSError::InvalidArguments(usage.to_string()))
        };
        w.write_output_nl(result)?;
        Ok(())
    }
    fn todo_cmd(&mut self,_:&[&str],_:&str,w:&mut IOWrapper)->CSResult<()>{
        self.cdll.print_todo(w)?;
        Ok(())
    }
    fn yn_loop(&self,msg:String,w:&mut IOWrapper)->CSResult<bool>{
        w.read_yn(format!("{msg}\nContinue (y/n)? (n default)"))
    }
    fn save_to_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()!=2{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        if args[1]=="temp.sav"{
            return Err(CSError::InvalidArguments(concat!("File name '",SWF!(Temp),"' shouldn't be a file name because the file will be overwritten after this program exits.").to_string()))
        }
        use std::fs::File;
        use std::io::Write;
        if !self.yn_loop(format!("Writing to this file name, \"{}\", will be overwritten.",args[1]),w)?{
            return Ok(())
        };
        let mut to_file={ match File::create(args[1]){ Ok(file)=>file, Err(e)=>return ErrToCSErr!(e) } };
        let mut sorted:Box<_>=self.cdll.hashmap.iter().collect();
        sorted.sort_by(|kv1,kv2| kv1.0.cmp(kv2.0));
        for (_,csl) in sorted.iter(){
            return_if_std_error!{writeln!(to_file,"{}",csl.borrow().save_write_str())}
        }
        self.has_saved=true;
        Ok(())
    }
    pub fn save_temp(&mut self)->CSResult<()>{
        use std::fs::File;
        use std::io::Write;
        let mut to_file={ match File::create(SWF!(Temp)){ Ok(file)=>file, Err(e)=>return ErrToCSErr!(e) } };
        let mut sorted:Box<_>=self.cdll.hashmap.iter().collect();
        sorted.sort_by(|kv1,kv2| kv1.0.cmp(kv2.0));
        for (_,csl) in sorted.iter(){
            return_if_std_error!{writeln!(to_file,"{}",csl.borrow().save_write_str())}
        }
        Ok(())
    }
    fn load_from_cmd(&mut self,args:&[&str],usage:&str,w:&mut IOWrapper)->CSResult<()>{
        if args.len()!=2{
            return Err(CSError::InvalidArguments(usage.to_string()))
        }
        use std::fs::File;
        use std::io::BufReader;
        let from_file={ match File::open(args[1]){ Ok(file)=>file, Err(e)=>return ErrToCSErr!(e) } };
        if !self.has_saved{ //No prompt if no unsaved data.
            if !self.yn_loop(format!("All unsaved cube data in this program will be erased before loading this file {}.",args[1]),w)?{
                return Ok(())
            };
        }
        let mut bufread=BufReader::new(from_file);
        let mut str=String::new();
        return_if_std_error!{bufread.read_to_string(&mut str)}
        let parsed_str:Box<_>=str.split_whitespace().filter(|&str| str!=SWF!(N)&&str!=SWF!(T)&&str!=SWF!(FB)&&str!=SWF!(CT))
            .enumerate().filter_map(|(i,s)| if let 3=i%4{ None }else{ Some(s) }).collect(); //Remove fuse_tier and converts_to
        if parsed_str.len()%3!=0{
            return Err(CSError::ParseError("TUI::read_to_file",format!(concat!("Incorrect format <",SWF!(N)," N; ",SWF!(T)," I; ",SWF!(FB)," I; ",SWF!(CT)," ((N|N or N),)*(N|N or N); converts_to: (N)+;>, where N are cube names and I is an integer"))))
        }
        self.cdll.remove_all_cubes();
        let mut link_strs=Vec::new();
        for (i,cube_str) in parsed_str.chunks(3).enumerate(){
            let i=i+1;
            let Some(cube_name)=cube_str[0].strip_suffix(';') else{ return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: Missing semi-colon at field converts_to"))) };
            check_valid_cube_name(cube_name)?;
            let Some(tier_str)=cube_str[1].strip_suffix(';') else{ return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: Missing semi-colon at field tier"))) };
            let tier={ match tier_str.parse::<i32>(){ Ok(tier)=>{ tier } Err(e)=> return ErrToCSErr!(e) } };
            self.cdll.add(super::CubeStruct::new(cube_name.to_string(),tier))?;
            w.write_output_nl(format!("Cube \"{}\" added",cube_name))?;
            let Some(valid_fcs)=cube_str[2].strip_suffix(';') else{ return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: Missing semi-colon at field fused_by"))) };
            if valid_fcs.is_empty(){ continue }
            for fcs in valid_fcs.split(',').collect::<Box<_>>().iter(){
                let mut iter3=fcs.split('|');
                let (Some(fc1),fc2_opt,None)=(iter3.next(),iter3.next(),iter3.next()) else{ //Size can be either 1 or 2, but never 3+
                    return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: There should only be 1 or 2 cube names delimited with ,")))
                };
                match fc1.strip_suffix('?'){
                    Some(fc1)=>{
                        let None=fc2_opt else{
                            return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: There should only be 1 cube name ending with ?, or ?;")))
                        };
                        check_valid_cube_name(fc1)?;
                    },
                    None=>{
                        let Some(fc2)=fc2_opt else{
                            return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: There should only be 2 cube names delimited with one | ending with , or ;")))
                        };
                        check_valid_cube_name(fc1)?; 
                        check_valid_cube_name(fc2)?;
                    }
                }
                link_strs.push((cube_name,fc1,fc2_opt));
            }
        }
        for (cube_name,cube_1,cube_2_opt) in link_strs{
            self.cdll.point_to(cube_name.to_string())?;
            match cube_2_opt{
                Some(cube_2)=>{
                    self.cdll.link_at_p_fb_pair(cube_1.to_string(),cube_2.to_string(),w)?;
                    w.write_output_nl(format!("Successfully linked cube \"{cube_name}\" with pairs \"{cube_1}\" and \"{cube_2}\""))?;
                }
                None=>{
                    let cube_noq=&cube_1[..cube_1.len()-1]; //Without question mark.
                    self.cdll.link_at_p_fb_single(cube_noq.to_string(),w)?;
                    w.write_output_nl(format!("Successfully linked cube \"{cube_name}\" with single \"{cube_noq}\""))?;
                }
            }
        }
        w.write_output_nl("File successfully read.".to_string())?;
        self.has_saved=true;
        Ok(())
    }
    fn exit_cmd(&mut self,_:&[&str],_:&str,w:&mut IOWrapper)->CSResult<()>{
        if !self.has_saved && !self.yn_loop("There may be unsaved cube data not saved yet.".to_string(),w)? {
            return Ok(())
        }
        self.done_program=true;
        Ok(())
    }
    fn usage_cmd(&mut self,_:&[&str],_:&str,w:&mut IOWrapper)->CSResult<()>{
        w.write_output_nl("Format: <command arg1 arg2 ...>\n\
        + means more than 1 set of arguments can also be added\n\
        ? means that an argument is optional\n\
        Command may have more than 1 name delimited with |".to_string())?;
        let mut unique_usage_str=HashSet::<&str>::new(); //To only show usages of multiple same name commands once.
        let mut usage_str_box=self.hm_command.values().filter(|&&(_,s1,_)|unique_usage_str.insert(s1)).map(|&(_,s1,s2)|s1.to_string()+"\n\t"+s2).collect::<Box<_>>();
        usage_str_box.sort_unstable();
        w.write_output_nl(usage_str_box.iter().enumerate().fold(String::new(),|res,(u,s)|{
            res+s+if u!=usage_str_box.len()-1{"\n"}else{""}
        }))?;
        Ok(())
    }
    pub fn cube_keys(&self,filter_opt:Option<String>)->Box<[&String]>{
        let mut keys=self.cdll.hashmap.keys().filter(|s|{
            if let Some(filter_str)=&filter_opt{
                s.to_lowercase().contains(&filter_str.to_lowercase())
            }else{
                true
            }
        }).collect::<Box<_>>();
        keys.sort_unstable();
        keys
    }
    pub fn cube_count(&self,filter_opt:Option<String>)->usize{
        self.cdll.hashmap.keys().filter(|s|{
            if let Some(filter_str)=&filter_opt{
                s.to_lowercase().contains(&filter_str.to_lowercase())
            }else{
                true
            }
        }).count()
    }
    pub fn cube_exists(&self,key:&String)->bool{
        self.cdll.hashmap.contains_key(key)
    }
    pub fn not_found_cmd(&mut self,args:&[&str],_:&str,_:&mut IOWrapper)->CSResult<()>{
        Err(super::error::CSError::InvalidCommand(args[0].to_string()))
    }
    pub fn b_has_saved(&self)->bool{
        self.has_saved
    }
    #[cfg(test)]
    fn test_multiple_commands(&mut self,args:Box<[Box<[&str]>]>)->CSResult<()>{
        for args in args.iter(){
            println!("\x1b[1mReading command {args:?}\x1b[0m");
            let default_cmd=&(Self::not_found_cmd as super::commands::Commands,"","");
            if args.is_empty(){ continue }
            let command_unwrap_tup=self.hm_command.get(args[0]).unwrap_or(default_cmd);
            command_unwrap_tup.0(self
                ,args,command_unwrap_tup.1,&mut IOWrapper::Stdio(&mut std::io::stdout(),&mut std::io::stdin()))?
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests{
    #[test]
    fn link_test()->Result<(),super::CSError>{
        let mut tui_obj:super::TUI=Default::default();
        let args=
        "add a 2 b 0 c 4 d 8\nread_all\n\
        link a with b c\nread_all\n\
        fuse a with d c\nread_all\n\
        link a with c\nread_all\n\
        unlink b c in a\nread_all\n\
        unlink d c in a\nread_all\n\
        unlink c in a\nread_all\nexit"
            .split('\n').map(|command| command.split_whitespace().collect::<Box<_>>())
            .collect::<Box<_>>();
        tui_obj.test_multiple_commands(args)?;
        Ok(())
    }
}