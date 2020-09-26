# rxos

## Known issues
- [ ] Double exceptions need to be fixed, they do not properly trigger.
- [ ] LAPIC timer is still not working

## TODO
- [ ] Create a better guide on how to get this project up and running.
- [ ] Replace the current linked list allocator with a slab allocator. This is mostly done, but the slab allocator throws an error when trying to parse the DSDT table.
- [ ] Get started with SMP, so we can actually utilize the different cores in the CPU. This does require the LAPIC timer to work.

## Building
### Windows
`cargo build`
`cargo bootimage --target x86_64-rxos.json`

## Running
You can simply navigate into the `kernel` folder, and run `cargo run`
