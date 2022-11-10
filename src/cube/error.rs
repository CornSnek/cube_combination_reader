use std::fmt::Formatter;
pub type CSResult<T>=Result<T,CSError>;
pub enum CSError{
    InvalidCommand(String),
    NameSyntax(String),
    NonExistentName(&'static str,String),
    NullPointer(&'static str),
    SameStruct(&'static str,String),
    InvalidArguments(&'static str),
    DuplicateName(&'static str,String),
    ParseError(&'static str,String),
    EmptyValue(&'static str,String),
    OtherError(Box<dyn std::error::Error>) //To propagate std::error::Error errors
}
impl std::fmt::Display for CSError{
    fn fmt(&self,f:&mut Formatter)->Result<(),std::fmt::Error>{
            match self{
                Self::InvalidCommand(str)=>write!(f,"'{str}' is not a valid command name. Type 'usage' for valid commands."),
                Self::NameSyntax(str)=>write!(f,"Cube name {} can only contain the valid characters [0-9a-zA-Z()]+ or contain only ?.",str),
                Self::NonExistentName(_,str)=>write!(f,"Cube name(s) {} not found",str),
                Self::NullPointer(_)=>write!(f,"CubeDLL pointer does not point to anything"),
                Self::SameStruct(_,str)=>write!(f,"CubeDLL pointer linking points to the same cube name \"{}\" (disallowed)",str),
                Self::InvalidArguments(str)=>write!(f,"Invalid arguments given in program. Correct usage: \"{}\"",str),
                Self::DuplicateName(_,str)=>write!(f,"Cube name \"{}\" already exists",str),
                Self::ParseError(_,str)=>write!(f,"Unable to parse string from file. Reason: {}",str),
                Self::EmptyValue(_,str)=>write!(f,"Empty value for key(s) {}",str),
                Self::OtherError(e)=>std::fmt::Display::fmt(e,f)
            }
    }
}
impl std::fmt::Debug for CSError{
    fn fmt(&self, f: &mut std::fmt::Formatter)->std::fmt::Result {
        match self{
            Self::InvalidCommand(_) => write!(f, "{{ CSError type: InvalidCommand }}"),
            Self::NameSyntax(_) => write!(f, "{{ CSError type: CubeName }}"),
            Self::NonExistentName(func,_) => write!(f, "{{ CSError type: NonExistentName, function: {func} }}"),
            Self::NullPointer(func) => write!(f, "{{ CSError type: NullPointer, function: {func} }}"),
            Self::SameStruct(func,_) => write!(f, "{{ CSError type: SameStruct, function: {func} }}"),
            Self::InvalidArguments(_) => write!(f, "{{ CSError type: InvalidArguments }}"),
            Self::DuplicateName(func,_) => write!(f, "{{ CSError type: DuplicateName, function: {func} }}"),
            Self::ParseError(func,_) => write!(f, "{{ CSError type: ParseError, function: {func} }}"),
            Self::EmptyValue(func,_) => write!(f, "{{ CSError type: EmptyValue, function: {func} }}"),
            Self::OtherError(e)=>{ write!(f,"Rust Error: ")?; std::fmt::Debug::fmt(e,f) }
        }
    }
}