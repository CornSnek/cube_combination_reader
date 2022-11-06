use std::io::Read;
use super::error::CSError;
pub struct TUI{
    cdll:super::CubeDLL,
    done_program:bool    
}
macro_rules! Err_to_custom_Err{
    ($e:tt)=>{ Err(CSError::OtherError(Box::new($e))) }
}
macro_rules! return_if_error{
    ($p:expr)=>(if let Err(e)=$p{ return Err_to_custom_Err!(e) })
}
impl TUI{
    pub fn new()->Self{
        Self{cdll:Default::default(),done_program:false}
    }
    fn do_command(&mut self,args:&[&str])->Result<(),CSError>{
        macro_rules! do_print_error{
            ($($arg:tt)*)=>{ eprint!("\x1b[1;33m"); eprint!($($arg)*); eprintln!("\x1b[0m"); }
        }
        match args[0]{
            "add"=>{
                if args.len()%2==0{
                    return Err(CSError::InvalidArguments("TUI::do_command","<add ((cube_name) (Tier))+>"))
                }
                for ch in args[1..].chunks(2){
                    let is_tier_num=ch[1].parse::<i32>();
                    match is_tier_num{
                        Ok(tier_num)=>{ self.cdll.add(super::CubeStruct::new(ch[0].to_string(),tier_num))?; }
                        Err(e)=> return Err_to_custom_Err!(e)
                    }
                    println!("Cube \"{}\" added",ch[0]);
                }
            }
            "read"=>{
                if args.len()==1{
                    return Err(CSError::InvalidArguments("TUI::do_command","<read (cube_name)+>"))
                }
                for arg in &args[1..]{
                    self.cdll.point_to(arg.to_string())?;
                    self.cdll.get_info_p()?;
                }
            }
            "read_all"=>{
                self.cdll.get_info_cube_paths();
            }
            "link"=>{
                if args.len()!=5{
                    return Err(CSError::InvalidArguments("TUI::do_command","<link (cube_a) with (cube_b1) (cube_b2)>"))
                }
                self.cdll.point_to(args[1].to_string())?;
                self.cdll.link_at_p_fby(args[3].to_string(),args[4].to_string())?;
                println!("Successfully linked cube \"{}\" with \"{}\" and \"{}\"",args[1],args[3],args[4]);
            }
            "unlink"=>{
                if args.len()==1{
                    return Err(CSError::InvalidArguments("TUI::do_command","<unlink (cube_name)+>"))
                }
                for arg in &args[1..]{
                    self.cdll.point_to(arg.to_string())?;
                    if self.cdll.unlink_at_p_fby()?{
                        println!("Successfully unlinked cube \"{arg}\" from fused_by properties.");
                    }else{
                        println!("Cube \"{arg}\" was already unlinked from fused_by properties.")
                    }
                }
            }
            "exit"=>{
                self.done_program=true;
            }
            "write_to"=>{
                if args.len()!=2{
                    return Err(CSError::InvalidArguments("TUI::do_command","<write_to (file_name)>"))
                }
                self.write_to_file(args[1])?;
            }
            "read_from"=>{
                if args.len()!=2{
                    return Err(CSError::InvalidArguments("TUI::do_command","<read_from (file_name)>"))
                }
                self.read_to_file(args[1])?;
            }
            "clear_all"=>{
                use std::io::Write;
                loop{
                    print!("All cube data will be erased without saving. Continue? (y/n)\n> ");
                    return_if_error!{std::io::stdout().flush()}
                    let mut buf=String::new();
                    return_if_error!{std::io::stdin().read_line(&mut buf)}
                    let args:Box<_>=buf.split_whitespace().collect();
                    if args[0]=="y"{ break }else if args[0]=="n"{ return Ok(()) }
                    break;
                }
                self.cdll.remove_all_cubes();
            }
            "change_tier"=>{
                if args.len()!=3{
                    return Err(CSError::InvalidArguments("TUI::do_command","<change_tier (cube_name) (this_tier)>"))
                }
                self.cdll.point_to(args[1].to_string())?;
                let tier={ match args[2].parse::<i32>(){ Ok(tier)=>{ tier } Err(e)=> return Err_to_custom_Err!(e) } };
                self.cdll.change_tier_at_p(tier)?;
                println!("Tier changed to {tier} for cube \"{}\"",args[1]);
            }
            "usage"=>{
                println!("Usage: Write names of cubes and their tiers and fusions with other cubes.\n\
                + means that more than one set of arguments can be repeated enclosed in ()+ (Example: add cube1 0 cube2 1 cube3 3\n\
                Commands: <add ((cube_name) (Tier))+>,<read (cube_name)+>,<read_all>,<link (cube_a) with (cube_b1) (cube_b2)>\n\
                <unlink (cube_name)+>,<clear_all>,<change_tier (cube_name) (this_tier)>\n\
                <write_to (file_name)>,<read_from (file_name)>,<exit>");
            }
            invalid=>{
                do_print_error!("Command not found: \"{invalid}\". Type \"usage\" for proper commands.");
            }
        }
        Ok(())
    }
    fn write_to_file(&self,file_name:&str)->Result<(),CSError>{
        use std::fs::File;
        use std::io::Write;
        loop{
            print!("Writing to this file name, \"{file_name}\", will be overwritten. Continue? (y/n)\n> ");
            return_if_error!{std::io::stdout().flush()}
            let mut buf=String::new();
            return_if_error!{std::io::stdin().read_line(&mut buf)}
            let args:Box<_>=buf.split_whitespace().collect();
            if args[0]=="y"{ break }else if args[0]=="n"{ return Ok(()) }
            break;
        }
        let mut to_file={ match File::create(file_name){ Ok(file)=>file, Err(e)=>return Err_to_custom_Err!(e) } };
        let mut sorted:Box<_>=self.cdll.hashmap.iter().collect();
        sorted.sort_by(|kv1,kv2| kv1.0.cmp(kv2.0));
        for (_,csl) in sorted.iter(){
            return_if_error!{writeln!(to_file,"{}",csl.borrow().save_write_str())}
        }
        Ok(())
    }
    fn read_to_file(&mut self,file_name:&str)->Result<(),CSError>{
        use std::fs::File;
        use std::io::{Write,BufReader};
        loop{
            print!("All unsaved cube data in this program will be erased before loading this file {file_name}. Continue? (y/n)\n> ");
            return_if_error!{std::io::stdout().flush()}
            let mut buf=String::new();
            return_if_error!{std::io::stdin().read_line(&mut buf)}
            let args:Box<_>=buf.split_whitespace().collect();
            if args[0]=="y"{ break }else if args[0]=="n"{ return Ok(()) }
        }
        let from_file={ match File::open(file_name){ Ok(file)=>file, Err(e)=>return Err_to_custom_Err!(e) } };
        let mut bufread=BufReader::new(from_file);
        let mut str=String::new();
        return_if_error!{bufread.read_to_string(&mut str)}
        let parsed_str:Box<_>=str.split_whitespace().filter(|&str| str!="name:"&&str!="tier:"&&str!="fused_by:"&&str!="converts_to:")
            .enumerate().filter_map(|(i,s)| if let 3=i%4{ None }else{ Some(s) }).collect(); //Remove fuse_tier and converts_to
        if parsed_str.len()%3!=0{
            return Err(CSError::ParseError("TUI::read_to_file","Incorrect format <name: N; tier: I; fuse_tier: I; fused_by: N,N; converts_to: (N)+;>, where N are cube names and I is an integer"))
        }
        self.cdll.remove_all_cubes();
        let mut link_strs:Vec<[&str;3]>=Vec::new();
        for cube_str in parsed_str.chunks(3){
            let Some(cube_name)=cube_str[0].strip_suffix(';') else{ return Err(CSError::ParseError("TUI::read_to_file","Missing semi-colon at field converts_to")) };
            let Some(tier_str)=cube_str[1].strip_suffix(';') else{ return Err(CSError::ParseError("TUI::read_to_file","Missing semi-colon at field tier")) };
            let tier={ match tier_str.parse::<i32>(){ Ok(tier)=>{ tier } Err(e)=> return Err_to_custom_Err!(e) } };
            self.cdll.add(super::CubeStruct::new(cube_name.to_string(),tier))?;
            let Some(two_cubes)=cube_str[2].strip_suffix(';') else{ return Err(CSError::ParseError("TUI::read_to_file","Missing semi-colon at field fused_by")) };
            if let Some((cube_1,cube_2))=two_cubes.split_once(','){
                link_strs.push([cube_name,cube_1,cube_2]); //Link other cubes if it exists for this cube.
            }
            println!("Cube \"{}\" added",cube_name);
        }
        for [cube_name,cube_1,cube_2] in link_strs{
            self.cdll.point_to(cube_name.to_string())?;
            self.cdll.link_at_p_fby(cube_1.to_string(),cube_2.to_string())?;
            println!("Successfully linked cube \"{cube_name}\" with \"{cube_1}\" and \"{cube_2}\"");
        }
        println!("File successfully read.");
        Ok(())
    }
    pub fn program_loop(&mut self)->std::io::Result<()>{
        use std::io::Write;
        while !self.done_program{
            print!("\nType \"usage\" for commands. Type \"exit\" to exit the program\n> ");
            std::io::stdout().flush()?;
            let mut buf=String::new();
            std::io::stdin().read_line(&mut buf)?;
            let args:Box<_>=buf.split_whitespace().collect();
            if let Err(e)=self.do_command(&args){
                println!("\x1b[1;33mError has occured: {e:?}: {e} \x1b[0m");
            }
        }
        Ok(())
    }
}