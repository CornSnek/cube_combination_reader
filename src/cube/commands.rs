
use super::error::CSResult;
pub type CommandHashMap<'a,T>=std::collections::HashMap<&'a str,(Commands<'a,T>,&'a str, &'a str)>;
pub type Commands<'a,T>=fn(&mut T, &[&str], &'a str,&mut dyn std::io::Write)->CSResult<()>;