use crate::cfg_debug::CfgDebug;

pub trait Tag: CfgDebug {
    fn id(&self) -> &str;
}

impl Tag for String {
    fn id(&self) -> &str {
        self.as_str()
    }
}

impl Tag for &str {
    fn id(&self) -> &str {
        self
    }
}
