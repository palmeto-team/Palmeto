use crate::arch::arm64::assembly::interrupt;

pub mod arm64;

///
/// This routine will handle architecture specific initialization,
/// _start shouldn't call architecture-specific functions directly.
/// 
/// Rather it will go through this function,
/// which will then handle calling them.
///
pub fn init()
{
    //
    // IMPORTANT:
    //     We need to initialize interrupt handlers before interrupts are enabled
    //
    arm64::exception::handlers::init();
    
    unsafe 
    {
        interrupt::enable_interrupts();
    }
}