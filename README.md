# rxos

## Known issues
- [ ] Double exceptions need to be fixed, they do not properly trigger.

## TODO
- [ ] Replace the current linked list allocator with a slab allocator.

## Building
### Windows
`cargo build`
`cargo bootimage --target x86_64-rxos.json`

## Running
### Windows
`qemu-system-x86_64 -drive format=raw,file=target/x86_64-rxos/debug/bootimage-kernel.bin`
