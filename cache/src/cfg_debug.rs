#[cfg(feature = "tracing")]
pub trait CfgDebug: std::fmt::Debug {}

#[cfg(feature = "tracing")]
impl<T> CfgDebug for T where T: std::fmt::Debug {}

#[cfg(not(feature = "tracing"))]
pub trait CfgDebug {}

#[cfg(not(feature = "tracing"))]
impl<T> CfgDebug for T {}
