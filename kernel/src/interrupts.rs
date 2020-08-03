use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::idt::PageFaultErrorCode;

use pic8259_simple::ChainedPics;
use spin;

use crate::{print, println, gdt, hlt_loop, apic};

///////////////////////////////////////////////////////////////////////////////////////////////////
// PIC
///////////////////////////////////////////////////////////////////////////////////////////////////
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub const PIC_OFFSET: u8 = 32;

// pub static PICS: spin::Mutex<ChainedPics> =
//     spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn initialize_apic() {
    unsafe {
        apic::disable_pic();
        apic::enable_apic();

        crate::hardware::rtc::enable_rtc();

        // Default IRQs
        apic::ioapic_set_irq(0, 0, InterruptIndex::Timer.as_u8());
        apic::ioapic_set_irq(1, 0, InterruptIndex::Keyboard.as_u8());
        apic::ioapic_set_irq(8, 0, InterruptIndex::RTC.as_u8());
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_OFFSET,
    Keyboard = PIC_OFFSET + 1,

    RTC = PIC_OFFSET + 8,
    ACPI = PIC_OFFSET + 9,

    PrimaryATA = PIC_OFFSET + 14,
    SecondaryATA = PIC_OFFSET + 15,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// IDT
///////////////////////////////////////////////////////////////////////////////////////////////////
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Exceptions
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);

        // PIC interrupts
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::RTC.as_usize()].set_handler_fn(rtc_interrupt_handler);
        idt[InterruptIndex::ACPI.as_usize()].set_handler_fn(acpi_interrupt_handler);

        idt
    };
}

/// Initialization
pub fn init_idt() {
    IDT.load();
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Exception handlers
///////////////////////////////////////////////////////////////////////////////////////////////////
/// Breakpoint handler
extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    // unsafe { apic::send_apic_eoi(0); }
}

/// Double fault handler
extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Page fault handler
extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut InterruptStackFrame, error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// PIC handlers
///////////////////////////////////////////////////////////////////////////////////////////////////
/// Timer interrupt handler
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    print!(".");
    // unsafe {
    //     PICS.lock()
    //         .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    // }
    unsafe { apic::apic_send_eoi(0); }
}

/// Keyboard interrupt handler
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    unsafe { apic::apic_send_eoi(0); }

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    debug!("Keyboard interrupt!");

    // unsafe {
    //     PICS.lock()
    //         .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    // }

    // unsafe { apic::apic_send_eoi(0); }
}

extern "x86-interrupt" fn acpi_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    println!("ACPI INTERRUPT!");

    // unsafe {
    //     PICS.lock()
    //         .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    // }

    unsafe { apic::apic_send_eoi(0); }
}

extern "x86-interrupt" fn rtc_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    //TODO: Probably want to use this irq to increment a "tick" global variable
    //      This way, I can pretty easily implement a sleep function
    unsafe {
        apic::apic_send_eoi(0);

        use cpuio::{inb, outb};
        outb(0x0C, 0x70);
        inb(0x71);
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Test cases
///////////////////////////////////////////////////////////////////////////////////////////////////
#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

//TODO: Implement test case for double fault exceptions. See https://os.phil-opp.com/double-fault-exceptions/#a-stack-overflow-test
