use cpuio::{inb, outb};

pub unsafe fn enable_rtc() {
    x86_64::instructions::interrupts::disable();
    outb(0x8B, 0x70);
    let prev = inb(0x71);
    outb(0x8B, 0x70);
    outb(prev | 0x40, 0x71);
    x86_64::instructions::interrupts::enable();
}
