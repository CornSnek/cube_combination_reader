use std::{fmt::Formatter};
#[allow(dead_code)]
pub enum CSError{
    NonExistentName(&'static str,String),
    NullPointer(&'static str),
    SameStruct(&'static str,String),
    InvalidArguments(&'static str,&'static str),
    DuplicateName(&'static str,String),
    ParseError(&'static str,String),
    EmptyValue(&'static str,String),
    OtherError(Box<dyn std::error::Error>) //To propagate std::error::Error errors
}
impl std::fmt::Display for CSError{
    fn fmt(&self,f:&mut Formatter)->Result<(),std::fmt::Error>{
            match self{
                CSError::NonExistentName(_,str)=>write!(f,"Cube name(s) {} not found",str),
                CSError::NullPointer(_)=>write!(f,"CubeDLL pointer does not point to anything"),
                CSError::SameStruct(_,str)=>write!(f,"CubeDLL pointer linking points to the same cube name \"{}\" (disallowed)",str),
                CSError::InvalidArguments(_,str)=>write!(f,"Invalid arguments given in program. Correct usage: \"{}\"",str),
                CSError::DuplicateName(_,str)=>write!(f,"Cube name \"{}\" already exists",str),
                CSError::ParseError(_,str)=>write!(f,"Unable to parse string from file. Reason: {}",str),
                CSError::EmptyValue(_,str)=>write!(f,"Empty value for key(s) {}",str),
                CSError::OtherError(e)=>std::fmt::Display::fmt(e,f)
            }
    }
}
impl std::fmt::Debug for CSError{
    fn fmt(&self, f: &mut std::fmt::Formatter)->std::fmt::Result {
        match self{
            CSError::NonExistentName(func,_) => write!(f, "{{ CSError type: NonExistentName, function: {func} }}"),
            CSError::NullPointer(func) => write!(f, "{{ CSError type: NullPointer, function: {func} }}"),
            CSError::SameStruct(func,_) => write!(f, "{{ CSError type: SameStruct, function: {func} }}"),
            CSError::InvalidArguments(func,_) => write!(f, "{{ CSError type: InvalidArguments, function: {func} }}"),
            CSError::DuplicateName(func,_) => write!(f, "{{ CSError type: DuplicateName, function: {func} }}"),
            CSError::ParseError(func,_) => write!(f, "{{ CSError type: ParseError, function: {func} }}"),
            CSError::EmptyValue(func,_) => write!(f, "{{ CSError type: EmptyValue, function: {func} }}"),
            CSError::OtherError(e)=>{ write!(f,"Rust Error: ")?; std::fmt::Debug::fmt(e,f) }
        }
    }
}