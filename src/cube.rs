pub mod error;
mod tui;
pub use tui::TUI;
use std::{rc::Rc, cell::RefCell, collections::HashMap, fmt::{Display, Formatter}};
type Link<T>=Rc<RefCell<T>>;
struct CubeStruct{
    name:String,
    tier:i32,
    converts_to:HashMap<String,Link<CubeStruct>>,
    fused_by:Option<[Link<CubeStruct>;2]>
}
macro_rules! SAVE_WRITE_FORMAT{
    ()=>{ "name: {}; tier: {}; fused_by: {}; converts_to: {};" }
}
impl CubeStruct{
    fn new(name:String,tier:i32)->Self{
        Self{name,tier,converts_to:HashMap::new(),fused_by:None}
    }
    fn add_to(&mut self,other:&Link<Self>)->Result<(),error::CSError>{
        let None=self.converts_to.insert(other.borrow().name.clone(),other.clone()) else{
            return Err(error::CSError::Link("CubeStruct::add_to"))
        };
        Ok(())
    }
    fn add_by(&mut self,other:&Link<Self>,other2:&Link<Self>)->Result<(),error::CSError>{
        let None=self.fused_by else{
            return Err(error::CSError::Link("CubeStruct::add_by"))
        };
        let (other,other2)=if other.borrow().name<other2.borrow().name{(other,other2)}else{(other2,other)};//Swap by alphabetical order.
        self.fused_by=Some([other.clone(),other2.clone()]);
        Ok(())
    }
    fn save_write_str(&self)->String{
        let fb_str={
            let mut str=String::new();
            if let Some([csl1,csl2])=&self.fused_by{
                str.push_str(csl1.borrow().name.as_str());
                str.push(',');
                str.push_str(csl2.borrow().name.as_str());
            }
            str
        };
        let ct_str={
            let mut str=String::new();
            for (i,csl) in self.converts_to.values().enumerate(){
                str.push_str(csl.borrow().name.as_str());
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
        let con_by={
            let mut str=String::new();
            if let Some(csl)=&self.fused_by{
                str+="\""; str.push_str(csl[0].borrow().name.as_str()); str+="\"";
                str+=",\""; str.push_str(csl[1].borrow().name.as_str()); str+="\"";
                str
            }else{
                "(None)".to_string()
            }
        };
        write!(f,r#"CubeStruct{{ name:["{}"], tier:[{}], converts_to:[{}], fused_by:[{}] }}"#,self.name,self.tier,con_to,con_by)
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
            cs.fused_by=None;
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
    fn unlink_at_p_fby(&self)->Result<bool,error::CSError>{
        let Some(csl)=&self.pointer else{ return Err(error::CSError::NullPointer("CubeDLL::link_at_p_fby")) };
        let mut cs_p=csl.borrow_mut();
        if let Some([csl1,csl2])=&cs_p.fused_by.take(){
            csl1.borrow_mut().converts_to.remove(&cs_p.name);
            csl2.borrow_mut().converts_to.remove(&cs_p.name);
            Ok(true)
        }else{
            Ok(false)
        }
    }
    fn get_info_p(&self)->Result<(),error::CSError>{
        if let Some(csl)=&self.pointer{
            println!("Info of cube pointer: {}",csl.borrow());
            Ok(())
        }else{ unreachable!() }
    }
    fn get_info_cube_paths(&self){
        println!("Syntax: fused_by: [2 cubes to fuse] => [[this cube name]](tier) => converts_to: [cube name array]");
        for csl in self.hashmap.values(){
            let cs=csl.borrow();
            let mut cs_str=" => [[\"".to_string();
            cs_str.push_str(cs.name.as_str());
            cs_str.push_str("\"]](");
            cs_str.push_str(cs.tier.to_string().as_str());
            cs_str.push_str(") => ");
            match &cs.fused_by{
                Some(csl)=>{
                    let mut fb_str=String::new();
                    fb_str+="fused_by: [\"";
                    fb_str+=csl[0].borrow().name.as_str();
                    fb_str+="\",\"";
                    fb_str+=csl[1].borrow().name.as_str();
                    fb_str+="\"]";
                    cs_str.insert_str(0,fb_str.as_str());
                }
                None=>{ cs_str.insert_str(0,"fused_by: (None)"); }
            }
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
