use acpi::{
    handler::{AcpiHandler, PhysicalMapping},
};

use crate::println;

use crate::memory::BootInfoFrameAllocator;
use x86_64::structures::paging::{
    mapper,
    Mapper,
    OffsetPageTable,
    PageSize,
    Size4KiB,
    FrameAllocator,
    frame::PhysFrame,
    page::Page,
    PageTableFlags as Flags,
};
use x86_64::addr::{PhysAddr, VirtAddr};

use core::ptr::NonNull;
use alloc::alloc::{Layout, alloc, dealloc};

pub struct AcpiMemoryHandler;

impl AcpiHandler for AcpiMemoryHandler {
    unsafe fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        size: usize
    ) -> PhysicalMapping<T> {
        // `physical_address` might not be page aligned, so padding might be needed
        // The size of the allocated memory needs to be the same as or bigger than size_of::<T>()
        // `size` should contain the size of T in bytes, I think, so I'll simply allocate that

        println!("phys_addr: 0x{:x}", physical_address);

        let phys_addr = PhysAddr::new(physical_address as u64);
        let layout = Layout::from_size_align(size, 4096).expect("Failed to create layout!");
        let virt_ptr = alloc(layout);
        let virt = VirtAddr::from_ptr(virt_ptr);

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(virt_ptr.cast()).expect("Failed to create NonNull ptr!"),
            region_length: size,
            mapped_length: size,
        }
    }

    fn unmap_physical_region<T>(&mut self, region: PhysicalMapping<T>) {
        // Unmap the given physical region

        let phys_addr = region.physical_start;
        let virt_ptr = region.virtual_start.as_ptr();
        let size = region.region_length;

        let layout = Layout::from_size_align(size, 4096).expect("Failed to create layout!");
        unsafe {
            dealloc(virt_ptr.cast(), layout);
        }

        //Absolutely no clue what to do here lol
    }
}
