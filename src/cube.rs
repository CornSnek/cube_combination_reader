pub mod error;
mod tui;
pub use tui::TUI;
use std::{rc::Rc, cell::RefCell, collections::HashMap, fmt::{Display, Formatter}};
type Link<T>=Rc<RefCell<T>>;
struct CubeStruct{
    name:String,
    tier:i32,
    converts_to:HashMap<String,Link<CubeStruct>>,
    fused_by:HashMap<StringPairKey,[Link<CubeStruct>;2]> //TODO: Change to HashMap
}
#[derive(Eq, Hash, PartialEq)]
struct StringPairKey(String,String); //To swap strings alphabetically.
impl StringPairKey{
    fn new(s1:&String,s2:&String)->Self{
        if s1<s2{ Self(s1.clone(),s2.clone()) }else{ Self(s2.clone(),s1.clone()) }
    } 
}
macro_rules! SAVE_WRITE_FORMAT{
    ()=>{ "name: {}; tier: {}; fused_by: {}; converts_to: {};" }
}
impl CubeStruct{
    fn new(name:String,tier:i32)->Self{
        Self{name,tier,converts_to:HashMap::new(),fused_by:HashMap::new()}
    }
    fn add_to(&mut self,other:&Link<Self>)->Result<(),error::CSError>{
        let None=self.converts_to.insert(other.borrow().name.clone(),other.clone()) else{
            return Err(error::CSError::Link("CubeStruct::add_to"))
        };
        Ok(())
    }
    fn add_by(&mut self,other:&Link<Self>,other2:&Link<Self>)->Result<(),error::CSError>{
        let key=StringPairKey::new(&other.borrow().name,&other2.borrow().name);
        let None=self.fused_by.insert(key,[other.clone(),other2.clone()]) else{
            return Err(error::CSError::Link("CubeStruct::add_by"))
        };
        Ok(())
    }
    fn save_write_str(&self)->String{
        let fb_str={
            let mut str=String::new();
            for (i,k) in self.fused_by.keys().enumerate(){
                str.push_str(k.0.as_str());
                str.push('|');
                str.push_str(k.1.as_str());
                if i!=self.fused_by.len()-1{ str.push(','); }
            }
            str
        };
        let ct_str={
            let mut str=String::new();
            for (i,k) in self.converts_to.keys().enumerate(){
                str.push_str(k.as_str());
                if i!=self.converts_to.len()-1{ str.push(','); }
            }
            str
        };
        format!(SAVE_WRITE_FORMAT!(),self.name.as_str(),self.tier.to_string(),fb_str,ct_str)
    }
}
impl Display for CubeStruct{
    fn fmt(&self, f: &mut Formatter)->Result<(),std::fmt::Error>{
        let con_to=if self.converts_to.len()!=0{
            let mut str=String::new();
            for (i,csl) in self.converts_to.values().enumerate(){
                str.push('"');
                str+=csl.borrow().name.as_str();
                str.push('"');
                if i!=self.converts_to.len()-1{ str.push(','); }
            }
            str
        }else{ "(None)".to_string() };
        let fus_by={
            let mut str=String::new();
            for (i,k) in self.fused_by.keys().enumerate(){
                str.push_str(k.0.as_str());
                str.push('|');
                str.push_str(k.1.as_str());
                if i!=self.fused_by.len()-1{ str.push(','); }
            }
            str
        };
        write!(f,r#"CubeStruct{{ name:["{}"], tier:[{}], converts_to:[{}], fused_by:[{}] }}"#,self.name,self.tier,con_to,fus_by)
    }
}
/*
impl Drop for CubeStruct{
    fn drop(&mut self){
        println!("Dropped CubeStruct \"{}\"",self.name);
    }
}
*/
struct CubeDLL{
    pointer:Option<Link<CubeStruct>>,
    hashmap:HashMap<String,Link<CubeStruct>>
}
impl Default for CubeDLL{
    fn default()->Self{
        Self{pointer:None,hashmap:HashMap::new()}
    }
}
impl CubeDLL{
    fn add(&mut self,cs:CubeStruct)->Result<(),error::CSError>{
        let str=&cs.name.clone();
        let link=Rc::new(RefCell::new(cs));
        let None=self.hashmap.insert(str.clone(),link) else{
            return Err(error::CSError::DuplicateName("CubeDLL::add",str.clone()))
        };
        Ok(())
    }
    fn point_to(&mut self,name:String)->Result<(),error::CSError>{
        if let Some(csl)=self.hashmap.get(&name){
            self.pointer=Some(csl.clone());
            Ok(())
        }else{
            Err(error::CSError::NonExistantName("CubeDLL::point_to",name.clone()))
        }
    }
    fn remove_all_cubes(&mut self){
        for csl in self.hashmap.values(){ //Remove possible Rc circular references.
            let mut cs=csl.borrow_mut();
            cs.fused_by.clear();
            cs.converts_to.clear();
        }
        self.hashmap.clear();
        self.pointer=None;
    }
    fn link_at_p_fby(&self,cs_n1:String,cs_n2:String)->Result<(),error::CSError>{
        let Some(csl1)=self.hashmap.get(&cs_n1) else{
            return Err(error::CSError::NonExistantName("CubeDLL::link_at_p_fby",cs_n1.clone()))
        };
        let Some(csl2)=self.hashmap.get(&cs_n2) else{
            return Err(error::CSError::NonExistantName("CubeDLL::link_at_p_fby",cs_n2.clone()))
        };
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::link_at_p_fby")) };
        if csl1.borrow().name==csl_p.borrow().name||csl2.borrow().name==csl_p.borrow().name {
            return Err(error::CSError::SameStruct("CubeDLL::link_at_p_fby",csl_p.borrow().name.clone()))
        }
        csl_p.borrow_mut().add_by(csl1,csl2)?;
        csl1.borrow_mut().add_to(&csl_p)?;
        if csl1.borrow().name==csl2.borrow().name{ return Ok(()) } //Don't rewrite twice if csl2 is the same as csl1.
        csl2.borrow_mut().add_to(&csl_p)?;
        Ok(())
    }
    fn unlink_at_p_fby(&self)->Result<bool,error::CSError>{ //TODO: Change this.
        let Some(csl)=&self.pointer else{ return Err(error::CSError::NullPointer("CubeDLL::link_at_p_fby")) };
        csl.borrow_mut().fused_by.clear();
        Ok(true)
    }
    fn get_info_p(&self)->Result<(),error::CSError>{
        if let Some(csl)=&self.pointer{
            println!("Info of cube pointer: {}",csl.borrow());
            Ok(())
        }else{ unreachable!() }
    }
    fn get_info_cube_paths(&self){
        println!("Syntax: fused_by: [(2 cubes) array] => [[this cube name]](tier) => converts_to: [cube name array]");
        for csl in self.hashmap.values(){
            let cs=csl.borrow();
            let mut cs_str=String::new();
            cs_str+="fused_by: [";
            for (i,k) in cs.fused_by.keys().enumerate(){
                cs_str.push('"');
                cs_str.push_str(k.0.as_str());
                cs_str.push_str("\"|\"");
                cs_str.push_str(k.1.as_str());
                cs_str.push('"');
                if i!=cs.fused_by.len()-1{ cs_str.push(','); }
            }
            cs_str+="]";
            cs_str.push_str(" => [[\"");
            cs_str.push_str(cs.name.as_str());
            cs_str.push_str("\"]](");
            cs_str.push_str(cs.tier.to_string().as_str());
            cs_str.push_str(") => ");
            cs_str.push_str("converts_to: [");
            for (i,(csl_strs,_)) in cs.converts_to.iter().enumerate(){
                cs_str.push('"'); cs_str.push_str(csl_strs.as_str()); cs_str.push('"');
                if i!=cs.converts_to.len()-1{ cs_str.push(','); }
            }
            cs_str.push(']');
            println!("{cs_str}");
        }
    }
    fn change_tier_at_p(&self,to_this_tier:i32)->Result<(),error::CSError>{
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::change_tier_at_p")) };
        csl_p.borrow_mut().tier=to_this_tier;
        Ok(())
    }
}
impl Display for CubeDLL{
    fn fmt(&self, f: &mut Formatter)->Result<(),std::fmt::Error>{
        let p_str=if let Some(csl)=&self.pointer{
            let mut str="\"".to_string(); str+=csl.borrow().name.as_str(); str.push('"'); str
        }else{ "(None)".to_string() };
        let h_str={
            let mut h_str=String::new();
            for (i,csl) in self.hashmap.values().enumerate(){
                h_str.push('"');
                h_str+=csl.borrow().name.as_str();
                h_str.push('"');
                if i!=self.hashmap.len()-1{ h_str.push(','); }
            }
            h_str
        };
        write!(f,"CubeDLL{{ pointer:[{}], hashmap:[{}] }}",p_str,h_str)
    }
}
impl Drop for CubeDLL{
    fn drop(&mut self){
        self.remove_all_cubes();
    }
}
