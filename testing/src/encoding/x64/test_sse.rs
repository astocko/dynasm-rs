extern crate dynasmrt;

use std::ops::Deref;

use dynasmrt::{DynasmApi, DynasmLabelApi};

use encoding::ndisasm;
use encoding::x64::opspecs_from_opmap;
use encoding::x64::generate_test;

// change palign to palignr
// loop and in instructions
// vpermil2pd
// vpermil2ps
// vpermil2 instructions removed -- https://software.intel.com/en-us/blogs/2009/01/29/recent-intelr-avx-architectural-changes
// vucomisd
// vucomiss
// monitorx
// mwaitx
// vpermd
//

// instruction typos cvtpd2dS, palign


#[test]
fn addss() {
    let mut ops = dynasmrt::x64::Assembler::new();

    dynasm!(ops
        ; bextr rax, rax, DWORD 0x10
        ; vucomiss xmm2,xmm3,DWORD [rax+0x10]
        ; adc rax, BYTE 0x10
        ; fadd st0, st5
    );

    let buf = ops.finalize().unwrap();

    // for b in buf.iter() {
    //     print!("\\x{:02X}", b);
    // }

    // println!("");

    // for b in buf.iter() {
    //     print!("{:02X} ", b);
    // }

    // println!("");


    // let dynasm_str = "palignr ymm1, ymm2, 1";
    // let nasm_str = ndisasm(buf.deref());

    // println!("{}", dynasm_str);
    // println!("{}", nasm_str);

    // assert_eq!(dynasm_str.to_lowercase(), nasm_str);

    // let opspecs = opspecs_from_opmap();

    // for opspec in opspecs {
    //     generate_test(opspec);
    // }

    // let mut ops = dynasmrt::x64::Assembler::new();

    // dynasm!(ops
    //         ; xsetbv
    // );

    // let buf = ops.finalize().unwrap();

    // let dynasm_str = "xsetbv";
    // let nasm_str = ndisasm(buf.deref());

    // assert_eq!(dynasm_str.to_lowercase(), nasm_str);


}
