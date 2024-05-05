# General dev-notes

Running multi-arch is difficult. 

## Qemu

`qemu-aarch64` version 9.0.0 [seems to have an issue](https://gitlab.com/qemu-project/qemu/-/issues/2326).  
Use a previous version, 0.7.2 works, manifests as VDSO-image address-alignment being zero, which 
causes a div-by-zero. 
