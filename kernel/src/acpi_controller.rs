use acpi::{
    handler::{AcpiHandler, PhysicalMapping},
    Acpi, AcpiError,
};

use x86_64::{
    structures::paging::{
        mapper,
        PageTable,
        OffsetPageTable,
        Page,
        PageSize,
        PhysFrame,
        Mapper,
        Size4KiB,
        FrameAllocator,
        PageTableFlags as Flags,
    },
    VirtAddr,
    PhysAddr,
};

use crate::println;

use core::ptr::NonNull;
use alloc::alloc::{Layout, alloc, dealloc};

pub struct AcpiMemoryHandler {
    pub phys_mem_offset: u64,
}

pub struct AcpiController {
    pub phys_mem_offset: u64,
    pub acpi: Acpi,
}

impl AcpiController {
    // TODO: Error handling
    pub fn new(phys_mem_offset: u64) -> Result<Self, AcpiError> {
        let acpi_data = {
            let mut acpi_handler = AcpiMemoryHandler {
                phys_mem_offset: phys_mem_offset,
            };
            unsafe { acpi::search_for_rsdp_bios(&mut acpi_handler) }
        }?;

        Ok(Self {
            phys_mem_offset: phys_mem_offset,
            acpi: acpi_data,
        })
    }
}

impl AcpiHandler for AcpiMemoryHandler {
    unsafe fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        size: usize
    ) -> PhysicalMapping<T> {
        // `physical_address` might not be page aligned, so padding might be needed
        // The size of the allocated memory needs to be the same as or bigger than size_of::<T>()
        // `size` should contain the size of T in bytes, I think, so I'll simply allocate that

        let virtual_start = self.phys_mem_offset + physical_address as u64;

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: core::ptr::NonNull::new_unchecked(virtual_start as *mut u8 as *mut T),
            region_length: size,
            mapped_length: size,
        }
    }

    fn unmap_physical_region<T>(&mut self, region: PhysicalMapping<T>) {
        // Unmap the given physical region
    }
}
