///For format!(...) and to be consistent for saving/loading the same characters.
macro_rules! SWF{
    (SAVE_STR)=>{ concat!(SWF!(N)," {}; ",SWF!(T)," {}; ",SWF!(FB)," {}; ",SWF!(CT)," {};") };
    (N)=>{"n:"};
    (T)=>{"t:"};
    (FB)=>{"fb:"};
    (CT)=>{"ct:"};
    (Temp)=>{"temp.sav"};
}
macro_rules! return_if_std_error{
    ($p:expr)=>(if let Err(e)=$p{ return ErrToCSErr!(e) })
}
use error::CSError;
macro_rules! ErrToCSErr{
    ($e:tt)=>{ Err(CSError::OtherError(Box::new($e))) }
}
mod commands;
pub(crate) mod tui;
mod error;
pub mod fltk_ui;
use std::{rc::Rc, cell::RefCell, collections::HashMap, fmt::{Display, Formatter}, hash::Hash};
use fltk_ui::app_utils::*;
pub enum IOWrapper<'a>{
    Stdio(&'a mut std::io::Stdout,&'a mut std::io::Stdin),
    FltkOutput(&'a mut OutputContainer),
}
impl IOWrapper<'_>{
    fn write_output_nl(&mut self,output:String)->error::CSResult<()>{
        self.write_output(output+"\n")
    }
    fn write_output(&mut self,output:String)->error::CSResult<()>{
        match self{
            Self::Stdio(sout,_)=>{
                use std::io::Write;
                return_if_std_error!(sout.write(output.as_bytes()));
                return_if_std_error!(sout.flush());
            }
            Self::FltkOutput(oc)=>{
                use fltk::prelude::*;
                if let Some(ref mut oc_ow)=oc.ow{
                    let OutputWidget::MLO(mlo)=oc_ow else{
                        unreachable!("MultilineOutput should only output here.")
                    };
                    let old_value=mlo.value();
                    mlo.set_value(&(old_value+&output));
                    return Ok(())
                }
                let mut oc_ow=fltk::output::MultilineOutput::default();
                let old_value=oc_ow.value();
                oc_ow.set_value(&(old_value+&output));
                oc.ow=Some(OutputWidget::MLO(oc_ow));
            }
        }
        Ok(())
    }
    fn read_yn(&mut self,prompt:String)->error::CSResult<bool>{
        match self{
            Self::Stdio(sout,sin)=>{
                loop{
                    use std::io::Write;
                    return_if_std_error!(sout.write((prompt.clone()+"> ").as_bytes()));
                    return_if_std_error!(sout.flush());
                    let mut buf=String::new();
                    return_if_std_error!{sin.read_line(&mut buf)}
                    let args:Box<_>=buf.split_whitespace().collect();
                    if args.is_empty(){ return Ok(false) }
                    if args[0]=="y"{ return Ok(true) }else if args[0]=="n"{ return Ok(false) }
                }
            }
            Self::FltkOutput(..)=>{
                if let Some(choice)=fltk::dialog::choice2_default((prompt.clone()+"> ").as_str(),"Yes","No",""){
                    Ok(if choice==1{ false }else{ true })
                }else{
                    Ok(false)
                }
            }
        }
    }
}
type Link=Rc<RefCell<CubeStruct>>;
struct CubeStruct{
    name:String,
    tier:i32,
    converts_to:HashMap<String,Link>,
    fused_by:HashMap<FuseKey,[Link;2]>
}
///Don't construct enums directly due to different hash possibility. Use FuseKeys::new_* implementation functions instead.
#[derive(Eq, Hash, PartialEq, Clone, Debug)]
enum FuseKey{ Pair(String,String), Single(String) }
impl FuseKey{
    ///Call new_pair to rearrange alphabetically and make the hash the same.
    fn new_pair(s1:&String,s2:&String)->Self{
        if s1<s2{ Self::Pair(s1.clone(),s2.clone()) }else{ Self::Pair(s2.clone(),s1.clone()) }
    }
    fn new_single(s1:&String)->Self{
        Self::Single(s1.clone())
    }
    fn contains_key(&self,cmp_str:&String)->bool{
        match self{
            Self::Pair(s0,s1)=>{s0==cmp_str||s1==cmp_str},
            Self::Single(s)=>{s==cmp_str}
        }
    }
    fn into_rewrite_key(self,old_str:&String,to_str:&String)->Self{
        match self{
            Self::Pair(ref s0,ref s1)=>{ Self::new_pair(if s0==old_str{to_str}else{s0},if s1==old_str{to_str}else{s1}) },
            Self::Single(ref s)=>{ if s==old_str{ Self::new_single(to_str) }else{ self } }
        }
    }
}
impl Display for FuseKey{
    fn fmt(&self, f: &mut Formatter<'_>)->Result<(),std::fmt::Error> {
        match self{
            Self::Single(s)=>write!(f,"({s}?)"),
            Self::Pair(s1,s2)=>write!(f,"({s1}|{s2})")
        }
    }
}
impl CubeStruct{
    fn new(name:String,tier:i32)->Self{
        Self{name,tier:tier.max(-1),converts_to:HashMap::new(),fused_by:HashMap::new()}
    }
    fn add_to(&mut self,other:&Link,w:&mut IOWrapper)->CSResult<()>{
        let other_str=other.borrow().name.clone();
        if other_str=="?"{ return Ok(()) } //Don't populate with ? (Redundant).
        if self.converts_to.insert(other_str,other.clone()).is_some(){
            w.write_output_nl(format!("Cube \"{}\" has already been added for \"{}\" as a conversion",&other.borrow().name,self.name))?;
        };
        Ok(())
    }
    fn remove_to_maybe(&mut self,other:&Link,w:&mut IOWrapper)->CSResult<()>{
        if other.borrow().fused_by.keys().any(|fuse_key| fuse_key.contains_key(&self.name)){
            w.write_output_nl(format!("Cube \"{}\" cannot be removed for \"{}\" as a conversion (Contains other keys)",&other.borrow().name,self.name))?;
        }else if self.converts_to.remove(&other.borrow().name).is_none(){
            w.write_output_nl(format!("Cube \"{}\" has already been removed for \"{}\" as a conversion",&other.borrow().name,self.name))?;
        }
        Ok(())
    }
    fn add_fb_pair(&mut self,other:&Link,other2:&Link,w:&mut IOWrapper)->CSResult<()>{
        let key=FuseKey::new_pair(&other.borrow().name,&other2.borrow().name);
        if self.fused_by.insert(key,[other.clone(),other2.clone()]).is_some(){
            w.write_output_nl(format!("Cubes \"{}\" and \"{}\" have already been added as a key to \"{}\"",&other.borrow().name,&other2.borrow().name,self.name))?;
        };
        Ok(())
    }
    fn add_fb_single(&mut self,other:&Link,w:&mut IOWrapper)->CSResult<()>{
        let key=FuseKey::new_single(&other.borrow().name);
        if self.fused_by.insert(key,[other.clone(),other.clone()]).is_some(){ //Make it so that the array returns the same Rc twice for single FuseKeys
            w.write_output_nl(format!("Cube \"{}\" has already been added as a key to \"{}\"",&other.borrow().name,self.name))?;
        }
        Ok(())
    }
    fn remove_fb_key(&mut self,key:FuseKey,w:&mut IOWrapper)->CSResult<()>{
        if self.fused_by.remove(&key).is_some(){
            w.write_output_nl(format!("Fuse key has been deleted for cube {}",self.name))?;
        }else{
            w.write_output_nl(format!("Fuse key has already been deleted for cube {}",self.name))?;
        }
        Ok(())
    }
    fn merge_single_keys(&mut self,l1:&Link,l2:&Link,w:&mut IOWrapper)->CSResult<()>{
        let (n1,n2)=(&l1.borrow().name,&l2.borrow().name);
        self.fused_by.remove(&FuseKey::new_single(n1));
        self.fused_by.remove(&FuseKey::new_single(n2));
        if self.fused_by.insert(FuseKey::new_pair(n1,n2),[l1.clone(),l2.clone()]).is_some() {
            w.write_output_nl(format!("Cubes \"{}\" and \"{}\" have already been added as a key to \"{}\"",&l1.borrow().name,&l2.borrow().name,self.name))?;
        };
        Ok(())
    }
    fn save_write_str(&self)->String{
        let fb_str={
            let mut str=String::new();
            for (i,k) in self.fused_by.keys().enumerate(){
                match k{
                    FuseKey::Pair(k0,k1)=>{
                        str.push_str(k0.as_str());
                        str.push('|');
                        str.push_str(k1.as_str());
                    }
                    FuseKey::Single(k0)=>{
                        str.push_str(k0.as_str());
                        str.push('?');
                    }
                }
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
        format!(SWF!(SAVE_STR),self.name.as_str(),self.tier.to_string(),fb_str,ct_str)
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
                match k{
                    FuseKey::Pair(k0,k1)=>{
                        str.push('[');
                        str.push_str(k0.as_str());
                        str.push('|');
                        str.push_str(k1.as_str());
                        str.push(']');
                    }
                    FuseKey::Single(k0)=>{
                        str.push_str(k0.as_str());
                        str.push('?');
                    }
                }
                if i!=self.fused_by.len()-1{ str.push(','); }
            }
            str
        };
        write!(f,r#"CubeStruct{{ name:["{}"], tier:[{}], converts_to:[{}], fused_by:[{}] }}"#,self.name,self.tier,con_to,fus_by)
    }
}
/*
impl Drop for CubeStruct{
    ///Printing drop for debugging.
    fn drop(&mut self){
        println!("Dropped CubeStruct \"{}\"",self.name);
    }
}
*/
use error::CSResult;
struct CubeDLL{
    pointer:Option<Link>,
    hashmap:HashMap<String,Link>
}
impl Default for CubeDLL{
    fn default()->Self{
        Self{pointer:None,hashmap:HashMap::new()}
    }
}
impl CubeDLL{
    fn add(&mut self,cs:CubeStruct)->CSResult<()>{
        let str=&cs.name.clone();
        let link=Rc::new(RefCell::new(cs));
        let None=self.hashmap.insert(str.clone(),link) else{
            return Err(error::CSError::DuplicateName("CubeDLL::add",str.clone()))
        };
        Ok(())
    }
    fn point_to(&mut self,name:String)->CSResult<()>{
        let csl={
            match self.hashmap.get(&name){
                Some(csl) => csl,
                _ => {
                    if name=="?"{ //Add ? if not added yet.
                        self.add(CubeStruct::new("?".to_string(),-1))?;
                        self.hashmap.get(&"?".to_string()).unwrap()
                    }else{
                        return Err(error::CSError::NonExistentName("CubeDLL::point_to",name.clone()))
                    }
                }
            }
        };
        self.pointer=Some(csl.clone());
        Ok(())
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
    fn link_at_p_fb_pair(&self,cs_n1:String,cs_n2:String,w:&mut IOWrapper)->CSResult<()>{
        let Some(csl1)=self.hashmap.get(&cs_n1) else{
            return Err(error::CSError::NonExistentName("CubeDLL::link_at_p_fb_pair",cs_n1.clone()))
        };
        let Some(csl2)=self.hashmap.get(&cs_n2) else{
            return Err(error::CSError::NonExistentName("CubeDLL::link_at_p_fb_pair",cs_n2.clone()))
        };
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::link_at_p_fb_pair")) };
        if csl1.borrow().name==csl_p.borrow().name||csl2.borrow().name==csl_p.borrow().name {
            return Err(error::CSError::SameStruct("CubeDLL::link_at_p_fb_pair",csl_p.borrow().name.clone()))
        }
        if let Some(qcsl)=self.hashmap.get(&"?".to_string()){
            let cmp_key=FuseKey::new_pair(&csl1.borrow().name,&csl2.borrow().name);
            if csl_p.borrow().name!="?"{
                if qcsl.borrow_mut().fused_by.remove(&cmp_key).is_some(){
                    w.write_output_nl(format!("Cube FuseKey \"{cmp_key}\" has been removed for ?"))?;
                }
            }else{
                let mut key_exists=false;
                'found_key: for csl in self.hashmap.values(){
                    for fuse_key in csl.borrow().fused_by.keys(){
                        if fuse_key==&cmp_key{
                            w.write_output_nl(format!("Cube FuseKey \"{cmp_key}\" cannot be added for ? as it already exists as known."))?;
                            key_exists=true;
                            break 'found_key
                        }
                    }
                }
                if key_exists{ return Ok(()) }
            }
        }
        csl_p.borrow_mut().add_fb_pair(csl1,csl2,w)?;
        csl1.borrow_mut().add_to(&csl_p,w)?;
        if csl1.borrow().name==csl2.borrow().name{ return Ok(()) } //Don't rewrite twice if csl2 is the same as csl1.
        csl2.borrow_mut().add_to(&csl_p,w)?;
        Ok(())
    }
    fn link_at_p_fb_single(&self,cs_n:String,w:&mut IOWrapper)->CSResult<()>{
        let Some(csl)=self.hashmap.get(&cs_n) else{
            return Err(error::CSError::NonExistentName("CubeDLL::link_at_p_fb_single",cs_n.clone()))
        };
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::link_at_p_fb_pair")) };
        if csl.borrow().name==csl_p.borrow().name {
            return Err(error::CSError::SameStruct("CubeDLL::link_at_p_fb_single",csl_p.borrow().name.clone()))
        }
        if let Some(qcsl)=self.hashmap.get(&"?".to_string()){
            let cmp_key=FuseKey::new_single(&csl.borrow().name);
            if csl_p.borrow().name!="?"{
                if qcsl.borrow_mut().fused_by.remove(&cmp_key).is_some(){
                    w.write_output_nl(format!("Cube FuseKey \"{cmp_key}\" has been removed for ?"))?;
                }
            }else{
                let mut key_exists=false;
                'found_key: for csl in self.hashmap.values(){
                    for fuse_key in csl.borrow().fused_by.keys(){
                        if fuse_key==&cmp_key{
                            w.write_output_nl(format!("Cube FuseKey \"{cmp_key}\" cannot be added for ? as it already exists as known."))?;
                            key_exists=true;
                            break 'found_key
                        }
                    }
                }
                if key_exists{ return Ok(()) }
            }
        }
        csl_p.borrow_mut().add_fb_single(csl,w)?;
        csl.borrow_mut().add_to(&csl_p,w)?;
        Ok(())
    }
    fn unlink_at_p_fb(&self)->CSResult<()>{
        let Some(csl_p)=&self.pointer else{ return Err(error::CSError::NullPointer("CubeDLL::unlink_at_p_fb")) };
        csl_p.borrow_mut().fused_by.clear();
        Ok(())
    }
    fn unlink_at_p_fb_keys(&self,cs_n1:String,cs_opt:Option<String>,w:&mut IOWrapper)->CSResult<()>{
        let Some(csl_p)=&self.pointer else{ return Err(error::CSError::NullPointer("CubeDLL::unlink_at_p_fb_keys")) };
        let Some(csl1)=self.hashmap.get(&cs_n1) else{
            return Err(error::CSError::NonExistentName("CubeDLL::unlink_at_p_fused_by",cs_n1.clone()))
        };
        if csl1.borrow().name==csl_p.borrow().name {
            return Err(error::CSError::SameStruct("CubeDLL::unlink_at_p_fused_by",csl_p.borrow().name.clone()))
        }
        match cs_opt{
            Some(cs_n2)=>{
                let Some(csl2)=self.hashmap.get(&cs_n2) else{
                    return Err(error::CSError::NonExistentName("CubeDLL::unlink_at_p_fused_by",cs_n2.clone()))
                };
                if csl2.borrow().name==csl_p.borrow().name {
                    return Err(error::CSError::SameStruct("CubeDLL::unlink_at_p_fused_by",csl_p.borrow().name.clone()))
                }
                let key=FuseKey::new_pair(&csl1.borrow().name.to_string(),&csl2.borrow().name.to_string());
                csl_p.borrow_mut().remove_fb_key(key,w)?;
                csl1.borrow_mut().remove_to_maybe(&csl_p,w)?;
                if csl1.borrow().name==csl2.borrow().name{ return Ok(()) } //Don't rewrite twice if csl2 is the same as csl1.
                csl2.borrow_mut().remove_to_maybe(&csl_p,w)?;
            }
            None=>{
                let key=FuseKey::new_single(&csl1.borrow().name.to_string());
                csl_p.borrow_mut().remove_fb_key(key,w)?;
                csl1.borrow_mut().remove_to_maybe(&csl_p,w)?;
            }
        }
        Ok(())
    }
    ///Remove all references of this CubeStruct at pointer including the pointer as well.
    fn destroy_at_p(&mut self,w:&mut IOWrapper)->CSResult<()>{
        let Some(csl)=self.pointer.take() else{ return Err(error::CSError::NullPointer("CubeDLL::destroy_at_p")) };
        let mut cs=csl.borrow_mut();
        let cs_name=cs.name.clone();
        for cs_fbs in cs.fused_by.values(){
            cs_fbs[0].borrow_mut().converts_to.remove(&cs_name);
            cs_fbs[1].borrow_mut().converts_to.remove(&cs_name);
            w.write_output_nl(format!("Removing \"{}\" from cubes \"{}\" and \"{}\"",cs_name,cs_fbs[0].borrow().name,cs_fbs[1].borrow().name))?;
        }
        cs.fused_by.clear();
        for cs_ct in cs.converts_to.values(){
            let mut cs_ct_bm=cs_ct.borrow_mut();
            let mut keys_to_delete=Vec::new();
            for fuse_k in cs_ct_bm.fused_by.keys(){
                if fuse_k.contains_key(&cs_name){
                    match fuse_k{
                        FuseKey::Pair(k0,k1)=>{
                            w.write_output_nl(format!("Removing fuse key pair {{\"{}\",\"{}\"}} for \"{}\"",k0,k1,cs_ct_bm.name))?;
                        }
                        FuseKey::Single(k0)=>{
                            w.write_output_nl(format!("Removing fuse key {{\"{}\"}} for \"{}\"",k0,cs_ct_bm.name))?;
                        }
                    }
                    keys_to_delete.push(fuse_k.clone());
                }
            }
            for fuse_k in keys_to_delete{ cs_ct_bm.fused_by.remove(&fuse_k); }
        }
        cs.converts_to.clear();
        self.hashmap.remove(&cs_name);
        Ok(())
    }
    fn rename_at_p(&mut self,to_name:String,w:&mut IOWrapper)->CSResult<()>{
        let None=self.hashmap.get(&to_name) else{ return Err(error::CSError::DuplicateName("CubeDLL::rename_at_p",to_name)) };
        let Some(csl)=&self.pointer else{ return Err(error::CSError::NullPointer("CubeDLL::rename_at_p")) };
        let mut cs=csl.borrow_mut();
        let cs_old_name=cs.name.clone();
        cs.name=to_name.clone();
        for cs_fbs in cs.fused_by.values(){
            let Some(v0)=cs_fbs[0].borrow_mut().converts_to.remove(&cs_old_name) else{
                return Err(error::CSError::EmptyValue("CubeDLL::rename_at_p",cs_fbs[0].borrow().name.clone()))
            };
            cs_fbs[0].borrow_mut().converts_to.insert(to_name.clone(),v0);
            if cs_fbs[0].borrow().name.clone()!=cs_fbs[1].borrow().name.clone(){ //If same key name.
                let Some(v1)=cs_fbs[1].borrow_mut().converts_to.remove(&cs_old_name) else{
                    return Err(error::CSError::EmptyValue("CubeDLL::rename_at_p",cs_fbs[1].borrow().name.clone()))
                };
                cs_fbs[1].borrow_mut().converts_to.insert(to_name.clone(),v1);
            }
            w.write_output_nl(format!("Changing key name of \"{cs_old_name}\" to \"{to_name}\" for \"{}\" and \"{}\"",cs_fbs[0].borrow().name,cs_fbs[1].borrow().name))?;
        }
        for cs_ct in cs.converts_to.values(){
            let mut cs_ct_bm=cs_ct.borrow_mut();
            let mut keys_to_change=Vec::new();
            for fuse_k in cs_ct_bm.fused_by.keys(){
                if fuse_k.contains_key(&cs_old_name){
                    keys_to_change.push(fuse_k.clone()); //To extract value from old key to replace with new.
                    w.write_output_nl(format!("Changing key names of \"{cs_old_name}\" to \"{to_name}\" for cube \"{}\"",cs_ct_bm.name))?;
                }
            }
            for fuse_k_old in keys_to_change.iter(){
                let Some(v)=cs_ct_bm.fused_by.remove(fuse_k_old) else{
                    return Err(error::CSError::EmptyValue("CubeDLL::rename_at_p",cs_ct_bm.name.clone()))
                };
                cs_ct_bm.fused_by.insert(fuse_k_old.clone().into_rewrite_key(&cs_old_name,&to_name),v);
            }
        }
        let Some(v)=self.hashmap.remove(&cs_old_name) else{
            return Err(error::CSError::EmptyValue("CubeDLL::rename_at_p","CubeDLL Hashmap".to_string()))
        };
        self.hashmap.insert(to_name,v);
        Ok(())
    }
    fn get_info_p(&self,w:&mut IOWrapper)->CSResult<()>{
        if let Some(csl)=&self.pointer{
            w.write_output_nl(format!("Info of cube pointer: {}",csl.borrow()))?;
            for key in csl.borrow().converts_to.keys(){
                let csl=self.hashmap.get(key).unwrap();
                w.write_output_nl(format!("Associated fusions: {}",csl.borrow()))?;
            }
            Ok(())
        }else{ unreachable!("Shouldn't be accessed. Should use point_to()") }
    }
    fn get_info_cube_paths(&self,w:&mut IOWrapper)->CSResult<()>{
        w.write_output_nl(format!("Syntax: fused_by: [fuse key array (single/pair)] => [[this cube name]](tier) => converts_to: [cube name array]"))?;
        for csl in self.hashmap.values(){
            let cs=csl.borrow();
            let mut cs_str=String::new();
            cs_str+="fused_by: [";
            for (i,k) in cs.fused_by.keys().enumerate(){
                match k{
                    FuseKey::Pair(k0,k1)=>{
                        cs_str.push('"');
                        cs_str.push_str(k0.as_str());
                        cs_str.push_str("\"|\"");
                        cs_str.push_str(k1.as_str());
                        cs_str.push('"');
                        
                    }
                    FuseKey::Single(k0)=>{
                        cs_str.push('"');
                        cs_str.push_str(k0.as_str());
                        cs_str.push_str("\"?");
                    }
                }
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
            w.write_output_nl(format!("{cs_str}"))?;
        }
        Ok(())
    }
    fn change_tier_at_p(&self,to_this_tier:i32,w:&mut IOWrapper)->CSResult<()>{
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::change_tier_at_p")) };
        if to_this_tier < -1{ w.write_output_nl(format!("Tier changed to -1 instead of {to_this_tier}"))?; }
        csl_p.borrow_mut().tier=to_this_tier.max(-1);
        Ok(())
    }
    fn merge_keys_at_p(&self,cs_n1:String,cs_n2:String,w:&mut IOWrapper)->CSResult<()>{
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::change_tier_at_p")) };
        let Some(csl1)=self.hashmap.get(&cs_n1) else{
            return Err(error::CSError::NonExistentName("CubeDLL::merge_keys_at_p",cs_n1.clone()))
        };
        let Some(csl2)=self.hashmap.get(&cs_n2) else{
            return Err(error::CSError::NonExistentName("CubeDLL::merge_keys_at_p",cs_n2.clone()))
        };
        match (csl_p.borrow().fused_by.get(&FuseKey::new_single(&cs_n1)).is_none(),csl_p.borrow().fused_by.get(&FuseKey::new_single(&cs_n2)).is_none()){
            (true,true)=>Err(error::CSError::NonExistentName("CubeDLL::merge_keys_at_p",cs_n1+" and "+&cs_n2)),
            (true,false)=>Err(error::CSError::NonExistentName("CubeDLL::merge_keys_at_p",cs_n1)),
            (false,true)=>Err(error::CSError::NonExistentName("CubeDLL::merge_keys_at_p",cs_n2)),
            _=>{
                csl_p.borrow_mut().merge_single_keys(csl1,csl2,w)?;
                Ok(())
            }
        }
    }
    fn get_fusions_at_p(&self,w:&mut IOWrapper)->CSResult<()>{
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::change_tier_at_p")) };
        let str_cs=&csl_p.borrow().name;
        let mut count:usize=1;
        for csl_other in self.hashmap.values(){
            let csl_other_str=&csl_other.borrow().name;
            for fuse_key in csl_other.borrow().fused_by.keys(){
                if fuse_key.contains_key(str_cs){
                    w.write_output_nl(format!("{count}: {csl_other_str} made with {fuse_key}"))?;
                    count+=1;
                }
            }
        }
        Ok(())
    }
}
mod bt{
    use super::FuseKey;
    use super::IOWrapper;
    use super::error::CSResult;
    use std::collections::{HashMap, HashSet};
    #[derive(Default)]
    pub struct BuildTreeNode{
        parent:Option<String>,
        cubes:HashMap<String,Vec<FuseKey>>
    }
    impl BuildTreeNode{
        ///Returns false if already added and visited. Adds parent as the first string.
        pub(super) fn add_cube(&mut self,cube:&String)->bool{
            if let None=self.parent{ self.parent=Some(cube.clone()); }
            if self.cubes.get(cube).is_some(){ false }else{
                self.cubes.insert(cube.clone(),Default::default());
                true
            }
        }
        pub(super) fn add_fuse_key(&mut self,cube:&String,fuse_key:&FuseKey){
            if let Some(bt)=self.cubes.get_mut(cube){
                bt.push(fuse_key.clone());
            }else{ unreachable!(concat!("Shouldn't be here at ",line!())) }
        }
        pub(super) fn print_simple(&self,w:&mut IOWrapper)->CSResult<()>{
            let Some(ref parent_str)=self.parent else{ unreachable!(concat!("Shouldn't be here at ",line!())) };
            let mut visit_hs=HashSet::<String>::new();
            let mut build_vec=Vec::<(usize,Vec<String>)>::new();
            self.print_simple_recurse(&mut build_vec,0,parent_str,&mut visit_hs);
            build_vec.sort_by(|lhs,rhs|lhs.0.cmp(&rhs.0));
            w.write_output(build_vec.iter().map(|(tier,str_v)|format!("[{tier}] => {}\n",str_v.concat())).collect::<Box<_>>().concat())?;
            Ok(())
        }
        fn print_simple_recurse(&self,build_vec:&mut Vec<(usize,Vec<String>)>,sort_tier:usize,visit_str:&String,visit_hs:&mut HashSet::<String>){
            if !visit_hs.insert(visit_str.clone()){ return }
            let Some(vec_fuse_keys)=self.cubes.get(visit_str) else{ return };
            build_vec.push((sort_tier,vec![visit_str.to_owned()]));
            let this_i=build_vec.len()-1;
            if !vec_fuse_keys.is_empty(){ build_vec[this_i].1.push(" => ".to_owned()); }
            for (i,fuse_key) in vec_fuse_keys.iter().enumerate(){
                build_vec[this_i].1.push(fuse_key.to_string()+if i!=vec_fuse_keys.len()-1{", "}else{""});
                match fuse_key{
                    FuseKey::Pair(s0,s1)=>{
                        self.print_simple_recurse(build_vec,sort_tier+1,s0,visit_hs);
                        self.print_simple_recurse(build_vec,sort_tier+1,s1,visit_hs);
                    }
                    FuseKey::Single(s0)=>{
                        self.print_simple_recurse(build_vec,sort_tier+1,s0,visit_hs);
                    }
                }
            }
        }
        pub(super) fn print_tree(&self,w:&mut IOWrapper)->CSResult<()>{
            let Some(ref parent_str)=self.parent else{ unreachable!(concat!("Shouldn't be here at ",line!())) };
            let mut visit_hs=HashSet::<String>::new();
            let mut build_vec=Vec::<(usize,String)>::new();
            self.print_tree_recurse(&mut build_vec,0,parent_str,&mut visit_hs,parent_str.to_owned());
            build_vec.sort_by(|lhs,rhs|lhs.0.cmp(&rhs.0));
            w.write_output(build_vec.iter().map(|(_,str)|str.to_owned()+"\n").collect::<Box<_>>().concat())?;
            Ok(())
        }
        fn print_tree_recurse(&self,build_vec:&mut Vec<(usize,String)>,sort_tier:usize,visit_str:&String,visit_hs:&mut HashSet::<String>,concat_str:String){
            build_vec.push((sort_tier,concat_str.clone()));
            if !visit_hs.insert(visit_str.clone()){ return }
            let Some(vec_fuse_keys)=self.cubes.get(visit_str) else{ return };
            for fuse_key in vec_fuse_keys{
                match fuse_key{
                    FuseKey::Pair(s0,s1)=>{
                        self.print_tree_recurse(build_vec,sort_tier+1,s0,visit_hs,concat_str.clone()+"/"+s0);
                        self.print_tree_recurse(build_vec,sort_tier+1,s1,visit_hs,concat_str.clone()+"/"+s1);
                    }
                    FuseKey::Single(s0)=>{
                        self.print_tree_recurse(build_vec,sort_tier+1,s0,visit_hs,concat_str.clone()+"/"+s0);
                    }
                }
            }
        }
    }
}
impl CubeDLL{
    fn build_tree_at_p(&self,w:&mut IOWrapper,arg:&str)->CSResult<()>{
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::change_tier_at_p")) };
        let cs_str=csl_p.borrow().name.clone();
        if arg=="build_tree"{
            w.write_output_nl(format!("Cube Combinations to get {}",cs_str))?;
        }
        let mut btn:bt::BuildTreeNode=Default::default();
        self.build_tree_recurse(&cs_str,&mut btn);
        if arg=="build_tree"{
            btn.print_simple(w)?;
        }else{
            btn.print_tree(w)?;
        }
        Ok(())
    }
    fn build_tree_recurse(&self,visit_str:&String,btn:&mut bt::BuildTreeNode){
        if btn.add_cube(visit_str){
            let cs=self.hashmap.get(visit_str).unwrap().borrow();
            for fuse_key in cs.fused_by.keys(){
                btn.add_fuse_key(visit_str,fuse_key);
                match fuse_key{
                    FuseKey::Pair(s0,s1)=>{
                        self.build_tree_recurse(s0,btn);
                        self.build_tree_recurse(s1,btn);
                    }
                    FuseKey::Single(s0)=>{
                        self.build_tree_recurse(s0,btn);
                    }
                }
            }
        }
    }
    fn print_todo(&self,w:&mut IOWrapper)->CSResult<()>{
        for (str,csl) in self.hashmap.iter(){
            if str!="?"{
                let cs=csl.borrow();
                if cs.tier==-1{
                    w.write_output_nl(format!("{str} cube has tier -1"))?;
                }
                if cs.fused_by.is_empty()&&cs.converts_to.is_empty(){
                    w.write_output_nl(format!("{str} cube is an orphan (No links added)"))?;
                }
                for fuse_key in cs.fused_by.keys(){
                    if let &FuseKey::Single(_)=fuse_key{
                        w.write_output_nl(format!("{str} contains a single fuse key {fuse_key}"))?;
                    }
                }
            }
        }
        if let Some(qcsl)=self.hashmap.get("?"){
            w.write_output_nl(format!("Printing fusion keys to unknown '?' Cubes"))?;
            for fuse_key in qcsl.borrow().fused_by.keys(){
                w.write_output(format!("{}, ",fuse_key))?;
            }
            w.write_output_nl(format!("\n"))?;
        }
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