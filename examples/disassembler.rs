use cydia_substrate::disasm::Disassembler;

fn main() {
    println!("HDE64 Disassembler Example");

    let code: Vec<u8> = vec![
        0x48, 0x89, 0xe5,
        0x48, 0x83, 0xec, 0x10,
        0x89, 0x7d, 0xfc,
        0x48, 0x8b, 0x45, 0xf8,
        0xc9,
        0xc3,
    ];

    println!("Disassembling {} bytes of code\n", code.len());

    let mut offset = 0;
    while offset < code.len() {
        match Disassembler::disassemble(unsafe { code.as_ptr().add(offset) }) {
            Some(instr) => {
                println!("Offset {:04x}:", offset);
                println!("  Length: {} bytes", instr.len());
                println!("  Opcode: 0x{:02x}", instr.opcode());

                if instr.has_modrm() {
                    println!("  ModR/M: 0x{:02x}", instr.modrm());
                    println!("    mod: {}", instr.modrm_mod());
                    println!("    reg: {}", instr.modrm_reg());
                    println!("    r/m: {}", instr.modrm_rm());
                }

                if instr.is_rip_relative() {
                    println!("  RIP-relative addressing");
                }

                if instr.has_error() {
                    println!("  ERROR in instruction");
                }

                println!();
                offset += instr.len();
            }
            None => {
                println!("Failed to disassemble at offset {:04x}", offset);
                break;
            }
        }
    }

    let min_size = 5;
    let copied = Disassembler::copy_instructions(code.as_ptr(), min_size);
    println!("Copied {} bytes to meet minimum size of {}", copied.len(), min_size);
}
