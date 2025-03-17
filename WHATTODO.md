#build the app
cargo build --target x86_64-unknown-uefi
#copy the efi
cp target/x86_64-unknown-uefi/debug/GrapeOS.efi esp/efi/boot/bootx64.efi 
#run it on the vm
qemu-system-x86_64 -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd -drive format=raw,file=fat:rw:esp