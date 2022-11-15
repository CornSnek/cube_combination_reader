
use super::error::CSResult;
use super::IOWrapper;
pub type CommandHashMap<'a>=std::collections::HashMap<&'a str,(Commands,&'a str, &'a str)>;
pub type Commands=fn(&mut super::tui::TUI, &[&str],&str,&mut IOWrapper)->CSResult<()>;