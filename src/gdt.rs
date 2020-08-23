use lazy_static::lazy_static;
use x86_64::{
    instructions::{
        segmentation::set_cs,
        tables::load_tss,
    },
    structures::{
        tss::TaskStateSegment,
        gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector},
    },
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    /// Create a new Interrupt Stack Table (IST) inside a Task State
    /// Segment (TSS) for stack switching on exception.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };

    /// Create a Global Descriptor Table (GDT) for holding our TSS.
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

/// Specifies particular elements to load into a segment from
/// descriptor tables.
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Load the GDT.
pub fn init() {
    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}