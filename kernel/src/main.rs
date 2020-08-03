#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

//Test stuff
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use] extern crate log;
extern crate alloc;

use core::panic::PanicInfo;
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

use bootloader::{BootInfo, entry_point};

use kernel::{
    print,
    println,

    task::executor::Executor,
    task::Task,
    task::keyboard,

    threading::{
        self,
        thread::Thread,
        with_scheduler,
    },
};

///////////////////////////////////////////////////////////////////////////////////////////////////
// Error handling
///////////////////////////////////////////////////////////////////////////////////////////////////
/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Main function
///////////////////////////////////////////////////////////////////////////////////////////////////
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use kernel::memory::BootInfoFrameAllocator;
    use x86_64::{structures::paging::MapperAllSizes, VirtAddr};

    kernel::logger::init().expect("Failed to load the kernel logger!");

    debug!("Hello world!");

    let regions = boot_info.memory_map.iter();
    let addr_ranges = regions.map(|r| r.range.start_addr()..r.range.end_addr());
    let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
    let available_memory = frame_addresses.count() * 4;
    debug!("Memory available: {} KiB", available_memory);

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    kernel::memory::update_physical_memory_offset(phys_mem_offset.as_u64());
    {
        let mut mapper = kernel::memory::MAPPER.lock();
        *mapper = unsafe { Some(kernel::memory::init(phys_mem_offset)) };
        let mut frame_allocator = kernel::memory::FRAME_ALLOCATOR.lock();
        *frame_allocator = unsafe {
            Some(BootInfoFrameAllocator::init(&boot_info.memory_map))
        };
        debug!("Mapper and frame allocator created!");
    }

    let mut mapper = kernel::memory::MAPPER.lock();
    let mut frame_allocator = kernel::memory::FRAME_ALLOCATOR.lock();

    kernel::init();
    kernel::allocator::init_heap(mapper.as_mut().unwrap(), frame_allocator.as_mut().unwrap()).expect("Heap initialization failed!");

    let acpi_controller = kernel::acpi_controller::AcpiController::new(phys_mem_offset.as_u64());

    match acpi_controller {
        Ok(controller) => {
            debug!("Found ACPI data!");
            controller.debug_print();

            controller.get_cpu();
            kernel::apic::update_ioapic_addr(*controller.get_io_apic_addr().iter().next().expect("Failed to get the first IOAPIC addr!") as u64);
        },
        Err(err) => {
            debug!("Did not find ACPI data :(");
            debug!("Reason: {:?}", err);
        },
    }

    kernel::interrupts::initialize_apic();
    x86_64::instructions::interrupts::enable();

    #[cfg(test)]
    test_main();

    let idle_thread = Thread::create(idle_thread, 2, mapper.as_mut().unwrap(), frame_allocator.as_mut().unwrap()).unwrap();
    with_scheduler(|s| s.set_idle_thread(idle_thread));

    for _ in 0..10 {
        let thread = Thread::create(thread_entry, 2, mapper.as_mut().unwrap(), frame_allocator.as_mut().unwrap()).unwrap();
        with_scheduler(|s| s.add_new_thread(thread));
    }
    let thread =
        Thread::create_from_closure(|| thread_entry(), 2, mapper.as_mut().unwrap(), frame_allocator.as_mut().unwrap())
            .unwrap();
    with_scheduler(|s| s.add_new_thread(thread));

    // let keyboard_thread = Thread::create(thread_keyboard, 2, &mut mapper, &mut frame_allocator).unwrap();
    // with_scheduler(|s| s.add_new_thread(keyboard_thread));

    debug!("It did not crash!");
    // loop {}
    // kernel::hlt_loop();
    thread_entry();
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Simple threads
///////////////////////////////////////////////////////////////////////////////////////////////////
fn idle_thread() -> ! {
    loop {
        x86_64::instructions::hlt();
        threading::yield_now();
    }
}

fn thread_entry() -> ! {
    let thread_id = with_scheduler(|s| s.current_thread_id()).as_u64();
    for _ in 0..=thread_id {
        print!("{}", thread_id);
        x86_64::instructions::hlt();
        threading::yield_now();
    }
    threading::exit_thread();
}

fn thread_keyboard() -> ! {
    let thread_id = with_scheduler(|s| s.current_thread_id()).as_u64();

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    threading::exit_thread();
}
