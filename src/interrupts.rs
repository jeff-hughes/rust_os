use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use pic8259_simple::ChainedPics;
use spin::Mutex;
use lazy_static::lazy_static;

use crate::{print, println};
use crate::gdt;

lazy_static! {
    /// Creates an Interrupt Descriptor Table used to handle various
    /// CPU exceptions.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);

        idt
    };
}

/// Initializes the Interrupt Descriptor Table for handling exceptions.
pub fn init_idt() {
    IDT.load();
}

/// Handles breakpoint exceptions.
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: &mut InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Handles double faults.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Handles timer hardware interrupts.
extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: &mut InterruptStackFrame)
{
    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}


// Hardware interrupts ---------------------------------------------------------

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Abstraction for two Intel 8259 PIC (programmable interrupt controller)
/// chips chained together, which control hardware interrupts.
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

/// An index used to identify of which line of the PIC is sending
/// an interrupt.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    /// Converts InterruptIndex to u8.
    fn as_u8(self) -> u8 {
        self as u8
    }

    /// Converts InterruptIndex to usize.
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}


// TESTS -----------------------------------------------------------------------

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}