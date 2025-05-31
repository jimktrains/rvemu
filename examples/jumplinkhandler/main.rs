extern crate rvemu;

use rvemu::cpu;
use rvemu::cpu::{Cpu, FRegisters, JumpLinkHandler, XRegisters};
use rvemu::devices::dram::Dram;
use rvemu::exception::Exception;

pub const SPF_MIN_ADDR: u64 = 0x800_0000;
pub const SPF_FABS: u64 = 0x800_bc40;
pub const SPF_FCN: u64 = 0x800_0400;

pub const DRAM_BASE: u64 = 0x800_0000;
pub const DRAM_SIZE: u64 = 0xff;

struct SysProvided {}

const SPF_FCN_TARGET_VALUE: f64 = 654.321;

impl JumpLinkHandler for SysProvided {
    fn should_handle(&self, new_pc: u64) -> bool {
        new_pc >= SPF_MIN_ADDR
    }
    fn handle(&self, new_pc: u64, cpu: &Cpu) -> (XRegisters, FRegisters) {
        let xregs = cpu.xregs.clone();
        let mut fregs = cpu.fregs.clone();
        match new_pc {
            SPF_FABS => {
                let f0 = fregs.read(cpu::REG_FA0);
                fregs.write(cpu::REG_FA0, f0.abs());
            }
            SPF_FCN => {
                fregs.write(cpu::REG_FA1, SPF_FCN_TARGET_VALUE);
            }
            _ => {}
        }

        (xregs, fregs)
    }
}

fn main() -> Result<(), Exception> {
    #[rustfmt::skip]
    let prog: Vec<u8> = vec![
        //  80000000: f2000553                fmv.d.x fa0,zero
        0x53, 0x05, 0x00, 0xf2,
        //  80000004: 3fc000ef                jal     80000400 <fcn>
        0xef, 0x00, 0xc0, 0x3f,
    ];

    let jh = SysProvided {};
    let mut cpu = Cpu::new();
    cpu.with_jump_link_handler(Box::new(jh));
    let mut dram = Box::new(Dram::new(DRAM_SIZE));
    dram.initialize(prog);
    cpu.bus.mount(DRAM_BASE, dram);

    cpu.reset();
    cpu.pc = DRAM_BASE;
    cpu.xregs.write(cpu::REG_SP, DRAM_BASE + DRAM_SIZE);

    println!("instr: {:x?}", cpu.cycle()?);
    println!("instr: {:x?}", cpu.cycle()?);
    cpu.print_registers();

    println!(
        "Was the handler successful? {:?}",
        cpu.fregs.read(cpu::REG_FA1) == SPF_FCN_TARGET_VALUE
    );
    Ok(())
}
