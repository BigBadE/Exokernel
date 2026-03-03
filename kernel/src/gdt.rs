use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use spin::Lazy;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const STACK_SIZE: usize = 4096 * 5;

static mut DOUBLE_FAULT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
static mut KERNEL_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    // Set up the double fault stack (IST entry 0)
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        let stack_start = VirtAddr::from_ptr(&raw const DOUBLE_FAULT_STACK as *const u8);
        stack_start + STACK_SIZE as u64
    };

    // Set up the kernel stack for privilege level changes (Ring 3 -> Ring 0)
    tss.privilege_stack_table[0] = {
        let stack_start = VirtAddr::from_ptr(&raw const KERNEL_STACK as *const u8);
        stack_start + STACK_SIZE as u64
    };

    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    // Kernel segments (Ring 0)
    let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());

    // User segments (Ring 3)
    let user_data_selector = gdt.append(Descriptor::user_data_segment());
    let user_code_selector = gdt.append(Descriptor::user_code_segment());

    // TSS segment
    let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

    (gdt, Selectors {
        kernel_code_selector,
        kernel_data_selector,
        user_code_selector,
        user_data_selector,
        tss_selector,
    })
});

#[derive(Debug, Clone, Copy)]
pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();

    unsafe {
        // Load kernel code segment
        CS::set_reg(GDT.1.kernel_code_selector);

        // Load kernel data segments
        DS::set_reg(GDT.1.kernel_data_selector);
        ES::set_reg(GDT.1.kernel_data_selector);
        SS::set_reg(GDT.1.kernel_data_selector);

        // Load TSS
        load_tss(GDT.1.tss_selector);
    }
}

pub fn selectors() -> &'static Selectors {
    &GDT.1
}
