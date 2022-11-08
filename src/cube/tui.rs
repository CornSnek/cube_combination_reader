use std::io::Read;
use super::error::CSError;
macro_rules! ErrToCSErr{
    ($e:tt)=>{ Err(CSError::OtherError(Box::new($e))) }
}
macro_rules! return_if_error{
    ($p:expr)=>(if let Err(e)=$p{ return ErrToCSErr!(e) })
}
pub struct TUI{
    cdll:super::CubeDLL,
    done_program:bool    
}
mod commands{
    use super::{TUI,super::{CubeStruct,error::CSError}};
    type CommandsHashMap<'a>=std::collections::HashMap<&'a str,fn(&mut TUI,&[&str])->Result<(),CSError>>;
    pub fn get_commands_hashmap()->CommandsHashMap<'static>{
        let mut cmd_hm=CommandsHashMap::new();
        cmd_hm.insert("add",|tui,args|{
            if args.len()%2==0{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<add ((cube_name) (Tier))+>"))
            }
            for chunk in args[1..].chunks(2){
                let is_tier_num=chunk[1].parse::<i32>();
                match is_tier_num{
                    Ok(tier_num)=>{ tui.cdll.add(CubeStruct::new(chunk[0].to_string(),tier_num))?; }
                    Err(e)=> return ErrToCSErr!(e)
                }
                println!("Cube \"{}\" added",chunk[0]);
            }
            Ok(())
        });
        let rem_f=|tui:&mut TUI,args:&[&str]|{
            if args.len()==1{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<remove|drop|destroy (cube_name)+>"))
            }
            for cube_str in &args[1..]{
                tui.cdll.point_to(cube_str.to_string())?;
                tui.cdll.destroy_at_p()?;
                println!("Cube \"{cube_str}\" and all of its links have been destroyed.");
            }
            Ok(())
        };
        cmd_hm.insert("remove",rem_f);
        cmd_hm.insert("drop",rem_f);
        cmd_hm.insert("destroy",rem_f);
        cmd_hm.insert("rename",|tui,args|{
            if args.len()!=4||args[2]!="to"{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<rename (cube) to (new_name)>"))
            }
            tui.cdll.point_to(args[1].to_string())?;
            tui.cdll.rename_at_p(args[3].to_string())?;
            println!("Cube \"{}\" successfully renamed to \"{}\"",args[1],args[3]);
            Ok(())
        });
        cmd_hm.insert("read",|tui,args|{
            if args.len()==1{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<read (cube_name)+>"))
            }
            for arg in &args[1..]{
                tui.cdll.point_to(arg.to_string())?;
                tui.cdll.get_info_p()?;
            }
            Ok(())
        });
        cmd_hm.insert("read_all",|tui,_|{
            tui.cdll.get_info_cube_paths();
            Ok(())
        });
        let link_f=|tui:&mut TUI,args:&[&str]|{
            let n@(4|5)=args.len() else{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<link|fuse (cube_name) with (cube_1) (cube_2)?>"))
            };
            if args[2]!="with"{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<link|fuse (cube_name) with (cube_1) (cube_2)?>"))
            }
            tui.cdll.point_to(args[1].to_string())?;
            if n==4{
                tui.cdll.link_at_p_fb_single(args[3].to_string())?;
                println!("Successfully linked cube \"{}\" with \"{}\"",args[1],args[3]);
            }else{
                tui.cdll.link_at_p_fb_pair(args[3].to_string(),args[4].to_string())?;
                println!("Successfully linked cube \"{}\" with \"{}\" and \"{}\"",args[1],args[3],args[4]);
            }
            Ok(())
        };
        cmd_hm.insert("link",link_f.clone()); 
        cmd_hm.insert("fuse",link_f.clone());
        cmd_hm.insert("merge",|tui,args|{
            if args.len()!=5||args[3]!="in"{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<merge (cube_a) (cube_b) in (tui_cube)>"))
            }
            tui.cdll.point_to(args[4].to_string())?;
            tui.cdll.merge_keys_at_p(args[1].to_string(),args[2].to_string())?;
            println!("Successfully merged keys \"{}\" with \"{}\" for \"{}\"",args[1],args[2],args[4]);
            Ok(())
        });
        cmd_hm.insert("unlink",|tui,args|{
            let n@(4|5)=args.len() else{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<unlink (cube_1) (cube_2)? in (cube_name)>"))
            };
            if args[n-2]!="in"{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<unlink (cube_1) (cube_2)? in (cube_name)>"))
            }
            tui.cdll.point_to(args[n-1].to_string())?;
            tui.cdll.unlink_at_p_fb_keys(args[1].to_string(),if n==5{Some(args[2].to_string())}else{None})?;
            println!("Successfully removed for cube \"{}\"",args[n-1]);
            Ok(())
        });
        cmd_hm.insert("unlink_all",|tui,args|{
            if args.len()==1{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<unlink_all (cube_name)+>"))
            }
            for arg in &args[1..]{
                tui.cdll.point_to(arg.to_string())?;
                tui.cdll.unlink_at_p_fb()?;
                println!("Successfully unlinked cube \"{arg}\" from fused_by properties.");
            }
            Ok(())
        });
        cmd_hm.insert("exit",|tui,_|{
            tui.done_program=true;
            Ok(())
        });
        let save_f=|tui:&mut TUI,args:&[&str]|{
            if args.len()!=2{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<save_to (file_name)>"))
            }
            tui.write_to_file(args[1])?;
            Ok(())
        };
        cmd_hm.insert("save_to",save_f);
        cmd_hm.insert("write_to",save_f);
        cmd_hm.insert("load_from",|tui,args|{
            if args.len()!=2{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<load_from (file_name)>"))
            }
            tui.read_to_file(args[1])?;
            Ok(())
        });
        let rem_all_f=|tui:&mut TUI,_:&[&str]|{
            if tui.yn_loop(format!("All cube data will be erased without saving."))?{
                tui.cdll.remove_all_cubes();
                println!("All cubes in the program have been removed.");
            }
            Ok(())
        };
        cmd_hm.insert("remove_all",rem_all_f);
        cmd_hm.insert("destroy_all",rem_all_f);
        cmd_hm.insert("drop_all",rem_all_f);
        cmd_hm.insert("change_tier",|tui,args|{
            if args.len()!=3{
                return Err(CSError::InvalidArguments("get_commands_hashmap","<change_tier (cube_name) (this_tier)>"))
            }
            tui.cdll.point_to(args[1].to_string())?;
            let tier={ match args[2].parse::<i32>(){ Ok(tier)=>{ tier } Err(e)=> return ErrToCSErr!(e) } };
            tui.cdll.change_tier_at_p(tier)?;
            println!("Tier changed to {tier} for cube \"{}\"",args[1]);
            Ok(())
        });
        cmd_hm.insert("usage",|_,_|{
            println!("Usage: Write names of cubes and their tiers and fusions with other cubes.\n\
                + means that more than one set of arguments can be repeated enclosed in ()+ (Example: add cube1 0 cube2 1 cube3 3\n\
                Commands: <add ((cube_name) (Tier))+>,<remove|drop|destroy (cube_name)+>,<rename (cube_name) to (new_cube_name)>,<read (cube_name)+>,\n\
                <read_all>,<link|fuse (cube_name) with (cube_1) (cube_2)?>,\n\
                <merge (cube_a) (cube_b) in (this_cube)><unlink_all (cube_name)+>,<unlink (cube_1) (cube_2)? in (cube_name)>,\n\
                <remove_all|drop_all|destroy_all>,<change_tier (cube_name) (this_tier)>\n\
                <save_to|write_to (file_name)>,<load_from (file_name)>,<exit>");
            Ok(())
        });
        cmd_hm
    }
}
macro_rules! do_print_error{
    ($($arg:tt)*)=>{ eprint!("\x1b[1;33m"); eprint!($($arg)*); eprintln!("\x1b[0m"); }
}
impl TUI{
    pub fn new()->Self{
        Self{cdll:Default::default(),done_program:false}
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
    fn write_to_file(&self,file_name:&str)->Result<(),CSError>{
        use std::fs::File;
        use std::io::Write;
        if !self.yn_loop(format!("Writing to this file name, \"{file_name}\", will be overwritten."))?{
            return Ok(())
        };
        let mut to_file={ match File::create(file_name){ Ok(file)=>file, Err(e)=>return ErrToCSErr!(e) } };
        let mut sorted:Box<_>=self.cdll.hashmap.iter().collect();
        sorted.sort_by(|kv1,kv2| kv1.0.cmp(kv2.0));
        for (_,csl) in sorted.iter(){
            return_if_error!{writeln!(to_file,"{}",csl.borrow().save_write_str())}
        }
        Ok(())
    }
    fn read_to_file(&mut self,file_name:&str)->Result<(),CSError>{
        use std::fs::File;
        use std::io::BufReader;
        if !self.yn_loop(format!("All unsaved cube data in this program will be erased before loading this file {file_name}."))?{
            return Ok(())
        };
        let from_file={ match File::open(file_name){ Ok(file)=>file, Err(e)=>return ErrToCSErr!(e) } };
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
                if fc2_opt.is_none()&&fc1.strip_suffix('?').is_none(){
                    return Err(CSError::ParseError("TUI::read_to_file",format!("Line {i}: Missing question-mark for a single fuse key at field fused_by")))
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
        Ok(())
    }
    pub fn program_loop(&mut self)->std::io::Result<()>{
        use std::io::Write;
        let commands_hm=commands::get_commands_hashmap();
        while !self.done_program{
            print!("\nType \"usage\" for commands. Type \"exit\" to exit the program\n> ");
            std::io::stdout().flush()?;
            let mut buf=String::new();
            std::io::stdin().read_line(&mut buf)?;
            let args:Box<_>=buf.split_whitespace().collect();
            if let Some(command)=commands_hm.get(args[0]){
                if let Err(e)=command(self,&args){
                    do_print_error!("Error has occured: {e:?}: {e}");
                }
            }else{
                do_print_error!("Command not found: \"{}\". Type \"usage\" for proper commands",args[0]);
            }
        }
        Ok(())
    }
    #[cfg(test)]
    fn test_multiple_commands(&mut self,args:Box<[Box<[&str]>]>)->Result<(),CSError>{
        let commands_hm=commands::get_commands_hashmap();
        for cmd in args.iter(){
            println!("\x1b[1mReading command {cmd:?}\x1b[0m");
            if let Some(command)=commands_hm.get(cmd[0]){
                if let Err(e)=command(self,&cmd){
                    do_print_error!("Error has occured: {e:?}: {e}");
                }
            }else{
                do_print_error!("Command not found: \"{}\". Type \"usage\" for proper commands",cmd[0]);
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