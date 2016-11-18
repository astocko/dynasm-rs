extern crate dynasmrt;

use std::ops::Deref;

use dynasmrt::{DynasmApi, DynasmLabelApi};

use encoding::ndisasm;

fn strip_size(asm_str: &String) -> String {

    println!("{}", asm_str);

    let asm_str = asm_str.replace("byte ", "");
    let asm_str = asm_str.replace("dword ", "");
    let asm_str = asm_str.replace("qword ", "");
    let asm_str = asm_str.replace("tword ", "");
    let asm_str = asm_str.replace("oword ", "");
    let asm_str = asm_str.replace("word ", "");
    asm_str
}

fn conv_imm(asm_str: &String, sz: usize) -> String {
    let sz_str = 0x10 + sz;
    let sz_str = format!("0x{:02X}", sz_str);
    let asm_str = asm_str.replace("0x10", sz_str.as_str());
    asm_str
}

fn to_u8(arr: &[u8]) -> String {
    let mut s = String::new();

    for a in arr {
        s += format!("\\x{:02X}", a).as_str();
    }

    return s;
}

fn try_x87(ds: &str, ns: &str, buf: &String) {
    if ds.contains("st0,") {
        let tmp = ds.replace("st0,", "");
        if tmp == ns {
            assert_eq!(tmp, ns, "{}", buf);
            return;
        }
    }

    if ds.contains(",st0") {
        let tmp = ds.replace(",st0", "");
        if tmp == ns {
            assert_eq!(tmp, ns, "{}", buf);
            return;
        }
    }

    if ds.contains(",st0") {
        let tmp = ds.replace(",st0", "");
        let tmp2 = ns.replace("to ", "");
        if tmp == tmp2 {
            assert_eq!(tmp, tmp2, "{}", buf);
            return;
        }

    }



    if ns.contains("st1") {
        let tmp = ns.replace(" st1", "");
        if tmp == ds {
            assert_eq!(ds, tmp, "{}", buf);
            return;
        }
    }

    if ns.contains(",st0") {
        let tmp = ns.replace(",st0", "");
        if tmp == ns {
            assert_eq!(tmp, ns, "{}", buf);
            return;
        }
    }

    if ns.contains("o16") {
        let tmp = ns.replace("o16 ", "");
        let tmp2 = ds.replace("w [", " ");

        if tmp == tmp2 {
            assert_eq!(tmp, tmp2, "{}", buf);
        }
    }

    assert_eq!(ds, ns, "{}", buf);

}

include!(concat!(env!("OUT_DIR"), "/tests.rs"));
