#[cfg(feature = "aux")]
pub mod aux;

#[cfg(all(feature = "start", feature = "aux"))]
pub(crate) mod dynlink;

#[cfg(feature = "vdso")]
pub(crate) mod vdso;
