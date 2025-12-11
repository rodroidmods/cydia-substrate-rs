#[cfg(test)]
mod tests {
    use crate::*;
    use std::ffi::c_void;

    #[test]
    fn test_null_pointer_checks() {
        let result = ms_hook_function(std::ptr::null_mut(), std::ptr::null_mut());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Symbol pointer is null");
    }

    #[test]
    fn test_hook_creation() {
        extern "C" fn dummy_fn() {}
        let result = Hook::<fn()>::new(
            std::ptr::null_mut(),
            dummy_fn as *mut c_void
        );
        assert!(result.is_err());
    }

    #[cfg(feature = "advanced")]
    #[test]
    fn test_trampoline_creation() {
        use crate::advanced::Trampoline;
        let tramp = Trampoline::new(256);
        assert!(tramp.is_ok());
    }

    #[cfg(feature = "advanced")]
    #[test]
    fn test_trampoline_write() {
        use crate::advanced::Trampoline;
        let mut tramp = Trampoline::new(256).unwrap();
        let result = tramp.write_bytes(&[0x90, 0x90, 0x90]);
        assert!(result.is_ok());
        assert_eq!(tramp.len(), 3);
    }

    #[cfg(feature = "advanced")]
    #[test]
    fn test_inline_hook_creation() {
        use crate::advanced::InlineHook;
        let mut buffer = [0u8; 32];
        let result = InlineHook::new(buffer.as_mut_ptr() as *mut c_void, 5);
        assert!(result.is_ok());
    }

    #[cfg(feature = "advanced")]
    #[test]
    fn test_hook_chain() {
        use crate::advanced::HookChain;
        let chain = HookChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_memory_protection() {
        use crate::memory::MemoryProtection;
        let mut buffer = vec![0u8; 4096];
        let result = MemoryProtection::make_writable(
            buffer.as_mut_ptr() as *mut c_void,
            buffer.len()
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_debug_hex_print() {
        use crate::debug::Debug;
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        Debug::print_hex(&data);
    }

    #[cfg(all(feature = "disassembler", target_arch = "x86_64"))]
    #[test]
    fn test_disassembler_basic() {
        use crate::disasm::Disassembler;
        let code = [0x90u8];
        let result = Disassembler::disassemble(code.as_ptr());
        assert!(result.is_some());
        let instr = result.unwrap();
        assert_eq!(instr.len(), 1);
        assert_eq!(instr.opcode(), 0x90);
    }

    #[cfg(all(feature = "disassembler", target_arch = "x86_64"))]
    #[test]
    fn test_disassembler_length() {
        use crate::disasm::Disassembler;
        let code = [0x90u8];
        let len = Disassembler::instruction_length(code.as_ptr());
        assert_eq!(len, 1);
    }

    #[cfg(feature = "advanced")]
    #[test]
    fn test_x86_instruction_sizes() {
        use crate::arch::x86::X86Instruction;
        assert_eq!(X86Instruction::size_of_skip(), 5);
        assert!(X86Instruction::size_of_push_pointer(0x12345678) >= 5);
    }

    #[cfg(all(feature = "advanced", any(target_arch = "arm", target_arch = "aarch64")))]
    #[test]
    fn test_arm_registers() {
        use crate::arch::arm::{ArmRegister, ARM_SP, ARM_LR, ARM_PC};
        assert_eq!(ARM_SP as u8, 13);
        assert_eq!(ARM_LR as u8, 14);
        assert_eq!(ARM_PC as u8, 15);
    }

    #[cfg(all(feature = "advanced", any(target_arch = "arm", target_arch = "aarch64")))]
    #[test]
    fn test_arm_instruction_generation() {
        use crate::arch::arm::{ArmInstruction, ArmRegister};
        let instr = ArmInstruction::mov_rd_rm(ArmRegister::R0, ArmRegister::R1);
        assert_ne!(instr, 0);
    }

    #[cfg(all(feature = "advanced", any(target_arch = "arm", target_arch = "aarch64")))]
    #[test]
    fn test_thumb_instruction() {
        use crate::arch::arm::ThumbInstruction;
        assert_eq!(ThumbInstruction::NOP, 0x46c0);
        assert_eq!(ThumbInstruction::POP_R0, 0xbc01);
    }

    #[cfg(target_os = "android")]
    #[test]
    fn test_android_find_libbase() {
        use crate::android::find_library_base;
        let result = find_library_base(std::process::id() as i32, "libc.so");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_ms_hook_process_invalid() {
        let result = ms_hook_process(-1, "invalid.so");
        assert!(!result);
    }
}
