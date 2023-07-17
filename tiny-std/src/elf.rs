#[cfg(feature = "aux")]
pub mod aux;

#[cfg(feature = "start")]
pub(crate) mod dynlink;

#[cfg(feature = "vdso")]
pub(crate) mod vdso;
