///For format!(...) and to be consistent for saving/loading the same characters.
macro_rules! SWF{
    (SAVE_STR)=>{ concat!(SWF!(N)," {}; ",SWF!(T)," {}; ",SWF!(FB)," {}; ",SWF!(CT)," {};") };
    (N)=>{"n:"};
    (T)=>{"t:"};
    (FB)=>{"fb:"};
    (CT)=>{"ct:"};
    (Temp)=>{"temp.sav"};
}
macro_rules! writeln_wrapper{
    ($($p:expr),*)=>{
        return_if_error!(writeln!($($p),*))
    }
}
macro_rules! write_wrapper{
    ($($p:expr),*)=>{
        return_if_error!(write!($($p),*))
    }
}
macro_rules! return_if_error{
    ($p:expr)=>(if let Err(e)=$p{ return ErrToCSErr!(e) })
}
use error::CSError;
macro_rules! ErrToCSErr{
    ($e:tt)=>{ Err(CSError::OtherError(Box::new($e))) }
}
mod commands;
mod tui;
mod error;
pub mod fltk_ui;
use std::{rc::Rc, cell::RefCell, collections::{HashMap, HashSet}, fmt::{Display, Formatter}, hash::Hash};
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
    fn add_to(&mut self,other:&Link,w:&mut dyn std::io::Write)->CSResult<()>{
        let other_str=other.borrow().name.clone();
        if other_str=="?"{ return Ok(()) } //Don't populate with ? (Redundant).
        if self.converts_to.insert(other_str,other.clone()).is_some(){
            writeln_wrapper!(w,"Cube \"{}\" has already been added for \"{}\" as a conversion",&other.borrow().name,self.name);
        };
        Ok(())
    }
    fn remove_to_maybe(&mut self,other:&Link,w:&mut dyn std::io::Write)->CSResult<()>{
        if other.borrow().fused_by.keys().any(|fuse_key| fuse_key.contains_key(&self.name)){
            writeln_wrapper!(w,"Cube \"{}\" cannot be removed for \"{}\" as a conversion (Contains other keys)",&other.borrow().name,self.name);
        }else if self.converts_to.remove(&other.borrow().name).is_none(){
            writeln_wrapper!(w,"Cube \"{}\" has already been removed for \"{}\" as a conversion",&other.borrow().name,self.name);
        }
        Ok(())
    }
    fn add_fb_pair(&mut self,other:&Link,other2:&Link,w:&mut dyn std::io::Write)->CSResult<()>{
        let key=FuseKey::new_pair(&other.borrow().name,&other2.borrow().name);
        if self.fused_by.insert(key,[other.clone(),other2.clone()]).is_some(){
            writeln_wrapper!(w,"Cubes \"{}\" and \"{}\" have already been added as a key to \"{}\"",&other.borrow().name,&other2.borrow().name,self.name);
        };
        Ok(())
    }
    fn add_fb_single(&mut self,other:&Link,w:&mut dyn std::io::Write)->CSResult<()>{
        let key=FuseKey::new_single(&other.borrow().name);
        if self.fused_by.insert(key,[other.clone(),other.clone()]).is_some(){ //Make it so that the array returns the same Rc twice for single FuseKeys
            writeln_wrapper!(w,"Cube \"{}\" has already been added as a key to \"{}\"",&other.borrow().name,self.name);
        }
        Ok(())
    }
    fn remove_fb_key(&mut self,key:FuseKey,w:&mut dyn std::io::Write)->CSResult<()>{
        if self.fused_by.remove(&key).is_some(){
            writeln_wrapper!(w,"Fuse key has been deleted for cube {}",self.name);
        }else{
            writeln_wrapper!(w,"Fuse key has already been deleted for cube {}",self.name);
        }
        Ok(())
    }
    fn merge_single_keys(&mut self,l1:&Link,l2:&Link,w:&mut dyn std::io::Write)->CSResult<()>{
        let (n1,n2)=(&l1.borrow().name,&l2.borrow().name);
        self.fused_by.remove(&FuseKey::new_single(n1));
        self.fused_by.remove(&FuseKey::new_single(n2));
        if self.fused_by.insert(FuseKey::new_pair(n1,n2),[l1.clone(),l2.clone()]).is_some() {
            writeln_wrapper!(w,"Cubes \"{}\" and \"{}\" have already been added as a key to \"{}\"",&l1.borrow().name,&l2.borrow().name,self.name);
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
impl Drop for CubeStruct{ //Printing drop for debugging.
    fn drop(&mut self){
        println!("Dropped CubeStruct \"{}\"",self.name);
    }
}*/
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
    fn link_at_p_fb_pair(&self,cs_n1:String,cs_n2:String,w:&mut dyn std::io::Write)->CSResult<()>{
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
                    writeln_wrapper!(w,"Cube FuseKey \"{cmp_key}\" has been removed for ?");
                }
            }else{
                let mut key_exists=false;
                'found_key: for csl in self.hashmap.values(){
                    for fuse_key in csl.borrow().fused_by.keys(){
                        if fuse_key==&cmp_key{
                            writeln_wrapper!(w,"Cube FuseKey \"{cmp_key}\" cannot be added for ? as it already exists as known.");
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
    fn link_at_p_fb_single(&self,cs_n:String,w:&mut dyn std::io::Write)->CSResult<()>{
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
                    writeln_wrapper!(w,"Cube FuseKey \"{cmp_key}\" has been removed for ?");
                }
            }else{
                let mut key_exists=false;
                'found_key: for csl in self.hashmap.values(){
                    for fuse_key in csl.borrow().fused_by.keys(){
                        if fuse_key==&cmp_key{
                            writeln_wrapper!(w,"Cube FuseKey \"{cmp_key}\" cannot be added for ? as it already exists as known.");
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
    fn unlink_at_p_fb_keys(&self,cs_n1:String,cs_opt:Option<String>,w:&mut dyn std::io::Write)->CSResult<()>{
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
    fn destroy_at_p(&mut self,w:&mut dyn std::io::Write)->CSResult<()>{
        let Some(csl)=self.pointer.take() else{ return Err(error::CSError::NullPointer("CubeDLL::destroy_at_p")) };
        let mut cs=csl.borrow_mut();
        let cs_name=cs.name.clone();
        for cs_fbs in cs.fused_by.values(){
            cs_fbs[0].borrow_mut().converts_to.remove(&cs_name);
            cs_fbs[1].borrow_mut().converts_to.remove(&cs_name);
            writeln_wrapper!(w,"Removing \"{}\" from cubes \"{}\" and \"{}\"",cs_name,cs_fbs[0].borrow().name,cs_fbs[1].borrow().name);
        }
        cs.fused_by.clear();
        for cs_ct in cs.converts_to.values(){
            let mut cs_ct_bm=cs_ct.borrow_mut();
            let mut keys_to_delete=Vec::new();
            for fuse_k in cs_ct_bm.fused_by.keys(){
                if fuse_k.contains_key(&cs_name){
                    match fuse_k{
                        FuseKey::Pair(k0,k1)=>{
                            writeln_wrapper!(w,"Removing fuse key pair {{\"{}\",\"{}\"}} for \"{}\"",k0,k1,cs_ct_bm.name);
                        }
                        FuseKey::Single(k0)=>{
                            writeln_wrapper!(w,"Removing fuse key {{\"{}\"}} for \"{}\"",k0,cs_ct_bm.name);
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
    fn rename_at_p(&mut self,to_name:String,w:&mut dyn std::io::Write)->CSResult<()>{
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
            writeln_wrapper!(w,"Changing key name of \"{cs_old_name}\" to \"{to_name}\" for \"{}\" and \"{}\"",cs_fbs[0].borrow().name,cs_fbs[1].borrow().name);
        }
        for cs_ct in cs.converts_to.values(){
            let mut cs_ct_bm=cs_ct.borrow_mut();
            let mut keys_to_change=Vec::new();
            for fuse_k in cs_ct_bm.fused_by.keys(){
                if fuse_k.contains_key(&cs_old_name){
                    keys_to_change.push(fuse_k.clone()); //To extract value from old key to replace with new.
                    writeln_wrapper!(w,"Changing key names of \"{cs_old_name}\" to \"{to_name}\" for cube \"{}\"",cs_ct_bm.name);
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
    fn get_info_p(&self,w:&mut dyn std::io::Write)->CSResult<()>{
        if let Some(csl)=&self.pointer{
            writeln_wrapper!(w,"Info of cube pointer: {}",csl.borrow());
            for key in csl.borrow().converts_to.keys(){
                let csl=self.hashmap.get(key).unwrap();
                writeln_wrapper!(w,"Associated fusions: {}",csl.borrow());
            }
            Ok(())
        }else{ unreachable!("Shouldn't be accessed. Should use point_to()") }
    }
    fn get_info_cube_paths(&self,w:&mut dyn std::io::Write)->CSResult<()>{
        writeln_wrapper!(w,"Syntax: fused_by: [fuse key array (single/pair)] => [[this cube name]](tier) => converts_to: [cube name array]");
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
            writeln_wrapper!(w,"{cs_str}");
        }
        Ok(())
    }
    fn change_tier_at_p(&self,to_this_tier:i32,w:&mut dyn std::io::Write)->CSResult<()>{
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::change_tier_at_p")) };
        if to_this_tier < -1{ writeln_wrapper!(w,"Tier changed to -1 instead of {to_this_tier}") }
        csl_p.borrow_mut().tier=to_this_tier.max(-1);
        Ok(())
    }
    fn merge_keys_at_p(&self,cs_n1:String,cs_n2:String,w:&mut dyn std::io::Write)->CSResult<()>{
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
    fn get_fusions_at_p(&self,w:&mut dyn std::io::Write)->CSResult<()>{
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::change_tier_at_p")) };
        let str_cs=&csl_p.borrow().name;
        let mut count:usize=1;
        for csl_other in self.hashmap.values(){
            let csl_other_str=&csl_other.borrow().name;
            for fuse_key in csl_other.borrow().fused_by.keys(){
                if fuse_key.contains_key(str_cs){
                    writeln_wrapper!(w,"{count}: {csl_other_str} made with {fuse_key}");
                    count+=1;
                }
            }
        }
        Ok(())
    }
    fn build_tree_at_p(&self,w:&mut dyn std::io::Write)->CSResult<()>{
        let mut hm_visited:HashSet<String>=HashSet::new();
        let Some(csl_p)=&self.pointer else {return Err(error::CSError::NullPointer("CubeDLL::change_tier_at_p")) };
        let cs_str=csl_p.borrow().name.clone();
        writeln_wrapper!(w,"Getting Combinations to get {}",cs_str);
        let mut build_str:Vec<(usize,String)>=Vec::new();
        self.build_tree_recurse(cs_str,&mut hm_visited,0,&mut build_str);
        build_str.sort_unstable(); //Print sorted by build_tier.
        writeln_wrapper!(w,"{}",build_str.into_iter().map(|i|i.1).collect::<Box<_>>().concat());
        Ok(())
    }
    ///Recursion over CubeStruct Links to get Cube Combinations for a single Cube. Asserting that the visit_str exists.
    fn build_tree_recurse(&self,visit_str:String,hm_visited:&mut HashSet<String>,build_tier:usize,build_str:&mut Vec<(usize,String)>){
        if hm_visited.get(&visit_str).is_none(){
            hm_visited.insert(visit_str.clone());
            let cs=self.hashmap.get(&visit_str).unwrap().borrow();
            for fuse_key in cs.fused_by.keys(){
                build_str.push((build_tier,format!("{build_tier}: {visit_str} made with {fuse_key}\n")));
                match fuse_key{
                    FuseKey::Pair(s0,s1)=>{
                        self.build_tree_recurse(s0.to_string(),hm_visited,build_tier+1,build_str);
                        self.build_tree_recurse(s1.to_string(),hm_visited,build_tier+1,build_str);
                    }
                    FuseKey::Single(s)=>{
                        self.build_tree_recurse(s.to_string(),hm_visited,build_tier+1,build_str);
                    }
                }
            }
        }
    }
    fn print_todo(&self,w:&mut dyn std::io::Write)->CSResult<()>{
        for (str,csl) in self.hashmap.iter(){
            if str!="?"{
                let cs=csl.borrow();
                if cs.tier==-1{
                    writeln_wrapper!(w,"{str} cube has tier -1");
                }
                if cs.fused_by.is_empty()&&cs.converts_to.is_empty(){
                    writeln_wrapper!(w,"{str} cube is an orphan (No links added)")
                }
                for fuse_key in cs.fused_by.keys(){
                    if let &FuseKey::Single(_)=fuse_key{
                        writeln_wrapper!(w,"{str} contains a single fuse key {fuse_key}");
                    }
                }
            }
        }
        if let Some(qcsl)=self.hashmap.get("?"){
            writeln_wrapper!(w,"Printing fusion keys to unknown '?' Cubes");
            for fuse_key in qcsl.borrow().fused_by.keys(){
                write_wrapper!(w,"{}, ",fuse_key);
            }
            writeln_wrapper!(w,"\n");
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