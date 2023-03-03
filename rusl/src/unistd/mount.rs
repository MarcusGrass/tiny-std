use crate::platform::FilesystemType;
use crate::string::unix_str::AsUnixStr;
use crate::Result;
use sc::syscall;

pub fn mount<SRC, TGT, DATA>(
    source: SRC,
    target: TGT,
    fs_type: FilesystemType,
    flags: u64,
    data: Option<DATA>,
) -> Result<()>
where
    SRC: AsUnixStr,
    TGT: AsUnixStr,
    DATA: AsUnixStr,
{
    unsafe {
        source.exec_with_self_as_ptr(|src| {
            target.exec_with_self_as_ptr(|tgt| {
                if let Some(data) = data {
                    data.exec_with_self_as_ptr(|data| {
                        let res = syscall!(
                            MOUNT,
                            src,
                            tgt,
                            fs_type.label().0.as_ptr(),
                            flags as usize,
                            data
                        );
                        bail_on_below_zero!(res, "`MOUNT` syscall failed");
                        Ok(res)
                    })
                } else {
                    let res = syscall!(
                        MOUNT,
                        src,
                        tgt,
                        fs_type.label().0.as_ptr(),
                        flags as usize,
                        0
                    );
                    bail_on_below_zero!(res, "`MOUNT` syscall failed");
                    Ok(res)
                }
            })
        })?;
    };
    Ok(())
}

pub fn unmount<TGT>(target: TGT) -> Result<()>
where
    TGT: AsUnixStr,
{
    target.exec_with_self_as_ptr(|ptr| {
        unsafe {
            let res = syscall!(UMOUNT2, ptr, 0);
            bail_on_below_zero!(res, "`UNMOUNT2` syscall failed");
        }
        Ok(())
    })
}
