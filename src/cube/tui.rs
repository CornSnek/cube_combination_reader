
macro_rules! ErrToCSErr{
    ($e:tt)=>{ Err(CSError::OtherError(Box::new($e))) }
}
macro_rules! return_if_error{
    ($p:expr)=>(if let Err(e)=$p{ return ErrToCSErr!(e) })
}
pub struct TUI{
    cdll:super::CubeDLL,
    done_program:bool,
    hm_command:commands::CommandHashMap<'static>,
    has_saved:bool
}
mod commands{
    use super::{TUI,super::error::CSResult};
    pub type CommandHashMap<'a>=std::collections::HashMap<&'a str,(Commands<'a>,&'a str, &'a str)>;
    pub type Commands<'a>=fn(&mut TUI, &[&str], &'a str)->CSResult<()>;
    pub fn get_commands_hashmap()->CommandHashMap<'static>{
        let mut cmd_hm=CommandHashMap::new();
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
}
macro_rules! do_print_error{
    ($($arg:tt)*)=>{ eprint!("\x1b[1;33m"); eprint!($($arg)*); eprintln!("\x1b[0m"); }
}
use std::collections::HashSet;
use std::io::Read;
use super::CubeStruct;
use super::error::CSError;
use super::error::CSResult;
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
///All functions ending in _cmd is for a hashmap.
impl TUI{
    pub fn new()->Self{
        let hm_command=commands::get_commands_hashmap();
        Self{cdll:Default::default(),
            done_program:false,
            hm_command,
            has_saved:true
        }
    }
    pub fn program_loop(&mut self){
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
                    let default_cmd=&(Self::not_found_cmd as commands::Commands,"","");
                    let command_unwrap_tup=self.hm_command.get(args[0]).unwrap_or(default_cmd);
                    if let Err(e)=command_unwrap_tup.0(self,&args,command_unwrap_tup.1){
                        do_print_error!("Error has occured: {e:?}: {e}");
                    }
                    match args[0]{
                        "destroy_all"|"drop_all"|"remove_all"|"save_to"|"write_to"|"load_from"| //These 6 implicitly do self.has_saved if y is entered.
                        "read"|"read_all"|"get_fusions"|"build_tree"| //These commands and below just reads all the data or do nothing to the data.
                        "find"|"search"|"usage"|"todo"|"exit"=>(),
                        _=>self.has_saved=false,
                    }
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
    fn add_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()%2==0||args.len()==1{
            return Err(CSError::InvalidArguments(usage))
        }
        for chunk in args[1..].chunks(2){
            check_valid_cube_name(chunk[0])?;
            if chunk[0]=="?"{ return Err(CSError::InvalidArguments("? cannot be added other than using it for linking")) }
            let is_tier_num=chunk[1].parse::<i32>();
            match is_tier_num{
                Ok(tier_num)=>{ self.cdll.add(CubeStruct::new(chunk[0].to_string(),tier_num))?; }
                Err(e)=> return ErrToCSErr!(e)
            }
            println!("Cube \"{}\" added",chunk[0]);
        }
        Ok(())
    }
    fn rm_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()==1{
            return Err(CSError::InvalidArguments(usage))
        }
        for cube_str in &args[1..]{
            if cube_str==&"?"{ return Err(CSError::InvalidArguments("? cannot be destroyed. Use unlink command to remove ? links instead.")) }
            check_valid_cube_name(cube_str)?;
            self.cdll.point_to(cube_str.to_string())?;
            self.cdll.destroy_at_p()?;
            println!("Cube \"{cube_str}\" and all of its links have been destroyed.");
        }
        Ok(())
    }
    fn rename_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()!=4||args[2]!="to"{
            return Err(CSError::InvalidArguments(usage))
        }
        if args[1]=="?"||args[3]=="?"{
            return Err(CSError::InvalidArguments("? cannot be renamed or be used as a name"))
        }
        check_valid_cube_name(args[1])?;
        check_valid_cube_name(args[3])?;
        self.cdll.point_to(args[1].to_string())?;
        self.cdll.rename_at_p(args[3].to_string())?;
        println!("Cube \"{}\" successfully renamed to \"{}\"",args[1],args[3]);
        Ok(())
    }
    fn read_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()==1{
            return Err(CSError::InvalidArguments(usage))
        }
        for arg in &args[1..]{
            check_valid_cube_name(arg)?;
            self.cdll.point_to(arg.to_string())?;
            self.cdll.get_info_p()?;
        }
        Ok(())
    }
    fn read_all_cmd(&mut self,_:&[&str],_:&'static str)->CSResult<()>{
        self.cdll.get_info_cube_paths();
        Ok(())
    }
    fn link_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        let n@(4|5)=args.len() else{
            return Err(CSError::InvalidArguments(usage))
        };
        if args[2]!="with"{
            return Err(CSError::InvalidArguments(usage))
        }
        check_valid_cube_name(args[1])?;
        self.cdll.point_to(args[1].to_string())?;
        check_valid_cube_name(args[3])?;
        if args[3]=="?"{ return Err(CSError::InvalidArguments("? cannot be used to fuse with another cube")) }
        if n==4{
            self.cdll.link_at_p_fb_single(args[3].to_string())?;
            println!("Successfully linked cube \"{}\" with \"{}\"",args[1],args[3]);
        }else{
            check_valid_cube_name(args[4])?;
            if args[4]=="?"{ return Err(CSError::InvalidArguments("? cannot be used to fuse with another cube")) }
            self.cdll.link_at_p_fb_pair(args[3].to_string(),args[4].to_string())?;
            println!("Successfully linked cube \"{}\" with \"{}\" and \"{}\"",args[1],args[3],args[4]);
        }
        Ok(())
    }
    fn merge_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()!=5||args[3]!="in"{
            return Err(CSError::InvalidArguments(usage))
        }
        check_valid_cube_name(args[1])?;
        if args[1]=="?"{ return Err(CSError::InvalidArguments("? cannot be used to merge")) }
        check_valid_cube_name(args[2])?;
        if args[2]=="?"{ return Err(CSError::InvalidArguments("? cannot be used to merge")) }
        check_valid_cube_name(args[4])?;
        self.cdll.point_to(args[4].to_string())?;
        self.cdll.merge_keys_at_p(args[1].to_string(),args[2].to_string())?;
        println!("Successfully merged keys \"{}\" with \"{}\" for \"{}\"",args[1],args[2],args[4]);
        Ok(())
    }
    fn unlink_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        let n@(4|5)=args.len() else{
            return Err(CSError::InvalidArguments(usage))
        };
        if args[n-2]!="in"{
            return Err(CSError::InvalidArguments(usage))
        }
        check_valid_cube_name(args[n-1])?;
        self.cdll.point_to(args[n-1].to_string())?;
        check_valid_cube_name(args[1])?;
        if args[1]=="?"{ return Err(CSError::InvalidArguments("? cannot be unlinked with another cube")) }
        if n==5 {
            check_valid_cube_name(args[2])?;
            if args[2]=="?"{ return Err(CSError::InvalidArguments("? cannot be unlinked with another cube")) }
        }
        self.cdll.unlink_at_p_fb_keys(args[1].to_string(),if n==5{ Some(args[2].to_string()) }else{ None })?;
        println!("Successfully removed for cube \"{}\"",args[n-1]);
        Ok(())
    }
    fn unlink_all_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()==1{
            return Err(CSError::InvalidArguments(usage))
        }
        for arg in &args[1..]{
            check_valid_cube_name(arg)?;
            self.cdll.point_to(arg.to_string())?;
            self.cdll.unlink_at_p_fb()?;
            println!("Successfully unlinked cube \"{arg}\"'s fused_by properties.");
        }
        Ok(())
    }
    fn change_tier_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()!=3{
            return Err(CSError::InvalidArguments(usage))
        }
        check_valid_cube_name(args[1])?;
        self.cdll.point_to(args[1].to_string())?;
        if args[1]=="?"{ return Err(CSError::InvalidArguments("? does not have a tier")) }
        let tier={ match args[2].parse::<i32>(){ Ok(tier)=>{ tier } Err(e)=> return ErrToCSErr!(e) } };
        self.cdll.change_tier_at_p(tier)?;
        Ok(())
    }
    fn rem_all_cmd(&mut self,_:&[&str],_:&'static str)->CSResult<()>{
        if self.yn_loop(format!("All cube data will be erased without saving."))?{
            self.cdll.remove_all_cubes();
            self.has_saved=true;
            println!("All cubes in the program have been removed.");
        }
        Ok(())
    }
    fn get_fusions_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()!=2{
            return Err(CSError::InvalidArguments(usage))
        }
        check_valid_cube_name(args[1])?;
        self.cdll.point_to(args[1].to_string())?;
        self.cdll.get_fusions_at_p()?;
        Ok(())
    }
    fn build_tree_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()!=2{
            return Err(CSError::InvalidArguments(usage))
        }
        check_valid_cube_name(args[1])?;
        self.cdll.point_to(args[1].to_string())?;
        self.cdll.build_tree_at_p()?;
        Ok(())
    }
    fn find_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()!=3{
            return Err(CSError::InvalidArguments(usage))
        }
        check_valid_cube_name(args[2])?;
        let result=match args[1]{
            "starts_with"=>{ //to_lowercase to be case-insensitive
                println!("Finding cube names starting with '{}'",args[2]);
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
                println!("Finding cube names containing substring '{}'",args[2]);
                let result=self.cdll.hashmap.keys().filter(|&k|
                    k.to_lowercase().contains(&args[2].to_lowercase()))
                .enumerate().fold(String::new(),|res,t|
                    res+&(t.0+1).to_string()+": "+t.1+"\n"
                );
                if !result.is_empty(){
                    result[..=result.len()-2].to_string()
                }else{"(None found)".to_string()}
            }
            _=>return Err(CSError::InvalidArguments(usage))
        };
        println!("{result}");
        Ok(())
    }
    fn todo_cmd(&mut self,_:&[&str],_:&'static str)->CSResult<()>{
        self.cdll.print_todo();
        Ok(())
    }
    fn yn_loop(&self,msg:String)->Result<bool,CSError>{
        use std::io::Write;
        loop{
            print!("{msg}\nContinue (y/n)? (n default)\n> ");
            return_if_error!{std::io::stdout().flush()}
            let mut buf=String::new();
            return_if_error!{std::io::stdin().read_line(&mut buf)}
            let args:Box<_>=buf.split_whitespace().collect();
            if args.is_empty(){ return Ok(false) }
            if args[0]=="y"{ return Ok(true) }else if args[0]=="n"{ return Ok(false) }
        }
    }
    fn save_to_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()!=2{
            return Err(CSError::InvalidArguments(usage))
        }
        if args[1]=="temp.sav"{
            return Err(CSError::InvalidArguments(concat!("File name '",SWF!(Temp),"' shouldn't be a file name because the file will be overwritten after this program exits.")))
        }
        use std::fs::File;
        use std::io::Write;
        if !self.yn_loop(format!("Writing to this file name, \"{}\", will be overwritten.",args[1]))?{
            return Ok(())
        };
        let mut to_file={ match File::create(args[1]){ Ok(file)=>file, Err(e)=>return ErrToCSErr!(e) } };
        let mut sorted:Box<_>=self.cdll.hashmap.iter().collect();
        sorted.sort_by(|kv1,kv2| kv1.0.cmp(kv2.0));
        for (_,csl) in sorted.iter(){
            return_if_error!{writeln!(to_file,"{}",csl.borrow().save_write_str())}
        }
        self.has_saved=true;
        Ok(())
    }
    fn save_temp(&mut self)->CSResult<()>{
        use std::fs::File;
        use std::io::Write;
        let mut to_file={ match File::create(SWF!(Temp)){ Ok(file)=>file, Err(e)=>return ErrToCSErr!(e) } };
        let mut sorted:Box<_>=self.cdll.hashmap.iter().collect();
        sorted.sort_by(|kv1,kv2| kv1.0.cmp(kv2.0));
        for (_,csl) in sorted.iter(){
            return_if_error!{writeln!(to_file,"{}",csl.borrow().save_write_str())}
        }
        Ok(())
    }
    fn load_from_cmd(&mut self,args:&[&str],usage:&'static str)->CSResult<()>{
        if args.len()!=2{
            return Err(CSError::InvalidArguments(usage))
        }
        use std::fs::File;
        use std::io::BufReader;
        let from_file={ match File::open(args[1]){ Ok(file)=>file, Err(e)=>return ErrToCSErr!(e) } };
        if !self.yn_loop(format!("All unsaved cube data in this program will be erased before loading this file {}.",args[1]))?{
            return Ok(())
        };
        let mut bufread=BufReader::new(from_file);
        let mut str=String::new();
        return_if_error!{bufread.read_to_string(&mut str)}
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
            println!("Cube \"{}\" added",cube_name);
            let Some(valid_fcs)=cube_str[2].strip_suffix(';') else{ return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: Missing semi-colon at field fused_by"))) };
            if valid_fcs.is_empty(){ continue }
            for fcs in valid_fcs.split(",").collect::<Box<_>>().iter(){
                let mut iter3=fcs.split('|');
                let (Some(fc1),fc2_opt,None)=(iter3.next(),iter3.next(),iter3.next()) else{ //Size can be either 1 or 2, but never 3+
                    return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: There should only be 2 cube names delimited with one | ending with , or ;")))
                };
                check_valid_cube_name(fc1)?;
                if fc2_opt.is_none()&&fc1.strip_suffix('?').is_none(){
                    return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: Missing question-mark for a single fuse key at field fused_by")))
                }else{
                    check_valid_cube_name(fc2_opt.unwrap())?;
                }
                link_strs.push((cube_name,fc1,fc2_opt));
            }
        }
        for (cube_name,cube_1,cube_2_opt) in link_strs{
            self.cdll.point_to(cube_name.to_string())?;
            match cube_2_opt{
                Some(cube_2)=>{
                    self.cdll.link_at_p_fb_pair(cube_1.to_string(),cube_2.to_string())?;
                    println!("Successfully linked cube \"{cube_name}\" with pairs \"{cube_1}\" and \"{cube_2}\"");
                }
                None=>{
                    let cube_noq=&cube_1[..cube_1.len()-1]; //Without question mark.
                    self.cdll.link_at_p_fb_single(cube_noq.to_string())?;
                    println!("Successfully linked cube \"{cube_name}\" with single \"{cube_noq}\"");
                }
            }
        }
        println!("File successfully read.");
        self.has_saved=true;
        Ok(())
    }
    fn exit_cmd(&mut self,_:&[&str],_:&'static str)->CSResult<()>{
        if !self.has_saved{
            if !self.yn_loop(format!("There may be unsaved cube data not saved yet."))?{
                return Ok(())
            }
        }
        self.done_program=true;
        Ok(())
    }
    fn usage_cmd(&mut self,_:&[&str],_:&'static str)->CSResult<()>{
        println!("Format: <command arg1 arg2 ...>\n\
        + means more than 1 set of arguments can also be added\n\
        ? means that an argument is optional\n\
        Command may have more than 1 name delimited with |");
        let mut unique_usage_str=HashSet::<&str>::new(); //To only show usages of multiple same name commands once.
        let mut usage_str_box=self.hm_command.values().filter(|&&(_,s1,_)|unique_usage_str.insert(s1)).map(|&(_,s1,s2)|s1.to_string()+"\n\t"+s2).collect::<Box<_>>();
        usage_str_box.sort_unstable();
        println!("{}",usage_str_box.iter().enumerate().fold(String::new(),|res,(u,s)|{
            res+s+if u!=usage_str_box.len()-1{"\n"}else{""}
        }));
        Ok(())
    }
    fn not_found_cmd(&mut self,args:&[&str],_:&str)->CSResult<()>{
        Err(super::error::CSError::InvalidCommand(args[0].to_string()))
    }
    #[cfg(test)]
    fn test_multiple_commands(&mut self,args:Box<[Box<[&str]>]>)->CSResult<()>{
        for args in args.iter(){
            println!("\x1b[1mReading command {args:?}\x1b[0m");
            let default_cmd=&(Self::not_found_cmd as commands::Commands,"","");
            let command_unwrap_tup=self.hm_command.get(args[0]).unwrap_or(default_cmd);
            if let Err(e)=command_unwrap_tup.0(self,&args,command_unwrap_tup.1){
                return Err(e)
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests{
    #[test]
    fn link_test()->Result<(),super::CSError>{
        let mut tui_obj=super::TUI::new();
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
        println!("a");
        tui_obj.test_multiple_commands(args)?;
        Ok(())
    }
}