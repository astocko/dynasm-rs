#[macro_use]
extern crate lazy_static;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

mod common {
    use std::{iter, slice};

    pub struct FormatStringIterator<'a> {
        inner: iter::Cloned<slice::Iter<'a, u8>>,
    }

    impl<'a> FormatStringIterator<'a> {
        pub fn new(buf: &'a [u8]) -> FormatStringIterator<'a> {
            FormatStringIterator { inner: buf.into_iter().cloned() }
        }
    }

    impl<'a> Iterator for FormatStringIterator<'a> {
        type Item = (u8, u8);

        fn next(&mut self) -> Option<(u8, u8)> {
            if let Some(ty) = self.inner.next() {
                let size = self.inner.next().expect("Invalid format string data");
                Some((ty, size))
            } else {
                None
            }
        }
    }
}

pub fn ndisasm(bytes: &[u8]) -> String {

    // echo -n -e '\xf3\x0f\x58\xc1' | ndisasm -b64 -

    let process = match Command::new("ndisasm")
        .args(&["-b", "64", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn() {
        Err(err) => panic!("couldn't spawn ndisasm: {}", err.description()),
        Ok(process) => process,
    };

    match process.stdin.unwrap().write_all(bytes) {
        Err(err) => panic!("Couldn't write to ndisasm stdin: {}", err.description()),
        Ok(_) => (),
    }

    let mut s = String::new();
    match process.stdout.unwrap().read_to_string(&mut s) {
        Err(err) => panic!("Couldn't read ndisasm stdout: {}", err.description()),
        Ok(_) => (),
    }

    let arr: Vec<&str> = s.split_at(28).1.split("\n").collect();
    return arr[0].trim().to_string();
}

mod testx64 {
    extern crate dynasm;

    use std::io::Write;
    use std::ops::Add;

    use self::dynasm::arch::x64::x64data;
    use self::dynasm::arch::x64::x64data::flags::*;

    use super::common::FormatStringIterator;



    #[derive(Clone, Copy, Debug)]
    pub enum ArgSize {
        Byte,
        Word,
        Dword,
        Qword,
        Oword,
        Hword,
        Auto,
        Any,
    }

    impl ArgSize {
        pub fn from_code(code: u8) -> ArgSize {
            match code as char {
                'b' => ArgSize::Byte,
                'w' => ArgSize::Word,
                'd' => ArgSize::Dword,
                'p' => ArgSize::Auto,
                'q' => ArgSize::Qword,
                'o' => ArgSize::Oword,
                'h' => ArgSize::Hword,
                '!' => ArgSize::Auto,
                '?' => ArgSize::Qword,
                _ => panic!("Unknown size: {:?} in from_code", code as char),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct OpSpec {
        pub mnemonic: &'static str,
        args: Vec<Vec<u8>>,
    }

    impl OpSpec {
        pub fn new(mnemonic: &'static str, args: Vec<Vec<u8>>) -> OpSpec {
            OpSpec {
                mnemonic: mnemonic,
                args: args,
            }
        }
    }


    pub fn new_sz_args<'a>(args: &[u8], r: u8) -> Vec<u8> {
        let mut args = args.to_owned();
        for a in args.iter_mut() {
            if *a == b'*' {
                *a = r
            }
        }
        args
    }



    pub fn opspecs_from_opmap<'a>() -> Vec<OpSpec> {
        let opmap_keys = x64data::mnemnonics();
        let mut instructions: Vec<OpSpec> = Vec::new();

        for key in opmap_keys {
            let mut argv = Vec::new();
            let opmap_data = x64data::get_mnemnonic_data(key);

            for data in opmap_data {
                for elem in data {
                    let mut args = elem.args;
                    let mut flags = elem.flags;

                    if args.contains(&b'*') {
                        if flags.intersects(AUTO_SIZE | AUTO_NO32 | AUTO_REXW | AUTO_VEXL) {
                            if flags.contains(AUTO_NO32) {
                                argv.push(new_sz_args(args, b'w'));
                                argv.push(new_sz_args(args, b'q'));
                            } else if flags.contains(AUTO_REXW) {
                                argv.push(new_sz_args(args, b'd'));
                                argv.push(new_sz_args(args, b'q'));
                            } else if flags.contains(AUTO_VEXL) {
                                argv.push(new_sz_args(args, b'o'));
                                argv.push(new_sz_args(args, b'h'));
                            } else {
                                argv.push(new_sz_args(args, b'w'));
                                argv.push(new_sz_args(args, b'd'));
                                // argv.push(new_sz_args(args, b'q'));
                            }
                        } else if flags.contains(PREF_66) {
                            argv.push(new_sz_args(args, b'o'));
                        }
                    } else {
                        argv.push(args.to_owned());
                    }
                }
            }

            instructions.push(OpSpec::new(key, argv));
        }

        instructions.sort();
        instructions
    }

    fn gen_gp8_reg(code: u8) -> &'static str {
        match code as char {
            'A' => "al",
            'B' => "cl",
            'C' => "dl",
            'D' => "bl",
            'E' => "spl",
            'F' => "bpl",
            'G' => "sil",
            'H' => "dil",
            'I' => "r8b",
            'J' => "r9b",
            'K' => "r10b",
            'L' => "r11b",
            'M' => "r12b",
            'N' => "r13b",
            'O' => "r14b",
            'P' => "r15b",
            _ => panic!("unknown gp16 reg"),
        }
    }

    fn gen_gp16_reg(code: u8) -> &'static str {
        match code as char {
            'A' => "ax",
            'B' => "cx",
            'C' => "dx",
            'D' => "bx",
            'E' => "sp",
            'F' => "bp",
            'G' => "si",
            'H' => "di",
            'I' => "r8w",
            'J' => "r9w",
            'K' => "r10w",
            'L' => "r11w",
            'M' => "r12w",
            'N' => "r13w",
            'O' => "r14w",
            'P' => "r15w",
            _ => panic!("unknown gp16 reg"),
        }
    }

    fn gen_gp32_reg(code: u8) -> &'static str {
        match code as char {
            'A' => "eax",
            'B' => "ecx",
            'C' => "edx",
            'D' => "ebx",
            'E' => "esp",
            'F' => "ebp",
            'G' => "esi",
            'H' => "edi",
            'I' => "r8d",
            'J' => "r9d",
            'K' => "r10d",
            'L' => "r11d",
            'M' => "r12d",
            'N' => "r13d",
            'O' => "r14d",
            'P' => "r15d",
            _ => panic!("unknown gp64 reg"),
        }
    }

    fn gen_gp64_reg(code: u8) -> &'static str {

        match code as char {
            'A' => "rax",
            'B' => "rcx",
            'C' => "rdx",
            'D' => "rbx",
            'E' => "rsp",
            'F' => "rbp",
            'G' => "rsi",
            'H' => "rdi",
            'I' => "r8",
            'J' => "r9",
            'K' => "r10",
            'L' => "r11",
            'M' => "r12",
            'N' => "r13",
            'O' => "r14",
            'P' => "r15",
            _ => panic!("unknown gp64 reg: {}", code as char),
        }
    }

    fn gen_gp_reg(code: u8, sz: ArgSize) -> (&'static str, ArgSize) {
        match sz {
            ArgSize::Byte => (gen_gp8_reg(code), sz),
            ArgSize::Word => (gen_gp16_reg(code), sz),
            ArgSize::Dword => (gen_gp32_reg(code), sz),
            ArgSize::Qword | ArgSize::Oword => (gen_gp64_reg(code), sz),
            ArgSize::Any => (gen_gp64_reg(code), ArgSize::Qword),
            _ => panic!("Unknown GP reg size: {:?}", sz),
        }
    }



    pub fn gen_seg_reg(code: u8, sz: ArgSize) -> (&'static str, ArgSize) {
        let reg = match code as char {
            'Q' => "es",
            'R' => "cs",
            'S' => "ss",
            'T' => "ds",
            'U' => "fs",
            'V' => "gs",
            _ => panic!("unknown segment register"),
        };

        (reg, sz)
    }

    pub fn gen_imm(sz: ArgSize) -> (&'static str, ArgSize) {
        let imm = match sz {
            ArgSize::Auto => "0x10",
            ArgSize::Byte => "BYTE 0x10",
            ArgSize::Word => "WORD 0x10",
            ArgSize::Dword => "DWORD 0x10",
            ArgSize::Qword => "QWORD 0x10",
            _ => "0x10",
        };
        (imm, sz)
    }

    pub fn gen_mem(sz: ArgSize) -> (&'static str, ArgSize) {
        let mut sz = sz;
        let mem = match sz {
            ArgSize::Auto => "[rax+0x10]",
            ArgSize::Byte => "BYTE [rax+0x10]",
            ArgSize::Word => "WORD [rax+0x10]",
            ArgSize::Dword => "DWORD [rax+0x10]",
            ArgSize::Qword => "QWORD [rax+0x10]",
            ArgSize::Oword => "OWORD [rax+0x10]",
            ArgSize::Hword => "HWORD [rax+0x10]",
            ArgSize::Any => "[rax+0x10]",
        };
        (mem, sz)
    }

    // pub fn gen_vsib32(sz: Option<char>) -> String {
    // }

    // pub fn gen_vsbi64() {}

    pub fn gen_legacy_reg(sz: ArgSize) -> (&'static str, ArgSize) {
        gen_gp_reg('A' as u8, sz)
    }

    pub fn gen_x87_reg(sz: ArgSize) -> (&'static str, ArgSize) {
        ("st1", sz)
    }

    pub fn gen_mmx_reg() -> (&'static str, ArgSize) {
        ("mmx0", ArgSize::Qword)
    }

    pub fn gen_sse_reg(sz: ArgSize) -> (&'static str, ArgSize) {
        let reg = match sz {
            ArgSize::Dword => "xmm0",
            ArgSize::Qword => "xmm1",
            ArgSize::Oword => "xmm2",
            ArgSize::Hword => "ymm3",
            ArgSize::Any => "xmm5",
            _ => panic!("unknown size sse reg: {:?}", sz),
        };

        (reg, sz)
    }

    pub fn gen_control_reg(sz: ArgSize) -> (&'static str, ArgSize) {
        ("cr8", ArgSize::Qword)
    }

    pub fn gen_debug_reg(sz: ArgSize) -> (&'static str, ArgSize) {
        ("dr2", ArgSize::Qword)
    }

    pub fn gen_mm_or_mem(sz: ArgSize) -> (&'static str, ArgSize) {
        ("mmx5", sz)
    }

    pub fn gen_sse_or_mem(sz: ArgSize) -> (&'static str, ArgSize) {
        gen_sse_reg(sz)
    }

    pub fn gen_cr8(sz: ArgSize) -> (&'static str, ArgSize) {
        ("cr8", ArgSize::Qword)
    }

    pub fn gen_ins_off(sz: ArgSize) -> (&'static str, ArgSize) {
        ("0x10", ArgSize::Byte)
    }

    pub fn gen_vsib(sz: ArgSize) -> (&'static str, ArgSize) {
        match sz {
            ArgSize::Oword => ("[xmm2*2]", sz),
            ArgSize::Hword => ("[ymm2*2]", sz),
            _ => ("[xmm2*2]", sz),
        }
    }


    pub fn generate_arg_str(args: &Vec<u8>) -> String {
        if args.len() > 0 {

            let mut args_string = String::new();

            for (code, sz) in FormatStringIterator::new(args) {
                let enc_size = ArgSize::from_code(sz);

                let (arg_string, _) = match code as char {
                    'A'...'P' => gen_gp_reg(code, enc_size),
                    'Q'...'V' => gen_seg_reg(code, enc_size),
                    'i' => gen_imm(enc_size),
                    'o' => gen_ins_off(enc_size),
                    'm' => gen_mem(enc_size),
                    'k' | 'l' => gen_vsib(enc_size),
                    'r' => gen_legacy_reg(enc_size),
                    'f' => gen_x87_reg(enc_size),
                    'x' => gen_mmx_reg(),
                    'y' => gen_sse_reg(enc_size),
                    's' => gen_seg_reg('Q' as u8, enc_size),
                    'c' => gen_control_reg(enc_size),
                    'd' => gen_debug_reg(enc_size),
                    'v' => gen_legacy_reg(enc_size), // or memory
                    'u' => gen_mm_or_mem(enc_size),
                    'w' => gen_sse_or_mem(enc_size),
                    'W' => gen_cr8(enc_size),
                    'X' => ("st0", enc_size),
                    _ => ("", ArgSize::Any),
                };

                args_string = args_string + arg_string + ",";
            }

            // if (args_string != "") {
            //     println!("{}", args_string);
            //     panic!("");
            // }

            let as_idx = args_string.len() - 1;
            args_string.remove(as_idx);
            args_string

        } else {
            "".to_string()
        }
    }

    pub fn generate_test<T: Write>(opspec: OpSpec, f: &mut T) {

        let test_tmpl = r#"
#[test]
fn {test_name}() {
    let mut ops = dynasmrt::x64::Assembler::new();

    dynasm!(ops
        ; {mnemonic}
    );

    let buf = ops.finalize().unwrap();

    let dynasm_str = "{mnemonic}";
    let nasm_str = ndisasm(buf.deref());
    let nasm_str = nasm_str.replace("yword", "hword");
    let nasm_str = nasm_str.replace(" +0x10", " 0x10");
    let dynasm_str = dynasm_str.replace("mmx", "mm");
    let mut dynasm_str = dynasm_str.to_lowercase();

    if dynasm_str == "xchg eax,eax" {
         dynasm_str = "nop".to_string();
    }

    if dynasm_str == nasm_str {
        assert_eq!(dynasm_str, nasm_str);
    } else if strip_size(&dynasm_str) == strip_size(&nasm_str) {
        assert_eq!(strip_size(&dynasm_str), strip_size(&nasm_str), "{}", to_u8(buf.deref()));
    } else {
        let ds = strip_size(&conv_imm(&dynasm_str, buf.deref().len()));
        let ns = strip_size(&nasm_str);
        // jump or call dword FIXME(astocko): ugly hack
        if ds.contains("st0") || ds.contains("st1") || ns.contains("st0") || ns.contains("st1") {
            try_x87(ds.trim(), ns.trim(), &to_u8(buf.deref()));
        } else {
            assert_eq!(ds.trim(), ns.trim(), "{}", to_u8(buf.deref()));
        }
    }

    // assert!(dynasm_str == nasm_str || strip_size(dynasm_str) == nasm_str);

    //assert_eq!(dynasm_str.to_lowercase(), nasm_str, "{:?}", buf.deref());
}
"#;

        let ins = opspec.mnemonic;

        if ins == "in" || ins == "loop" || ins == "mov" {
            return;
        }

        let mut x = 1;

        for arg in opspec.args {

            let test_name = format!("{}_{}", ins, x);
            x += 1;
            let arg_string = generate_arg_str(&arg);
            let mnem = format!("{} {}", ins, generate_arg_str(&arg));
            // let mnem = mnem.trim().to_string();

            // let mnem = mnem.replace("st0,", "");

            let mut test_str = test_tmpl.replace("{test_name}", test_name.as_str());
            let test_str = test_str.replace("{mnemonic}", mnem.as_str());

            f.write_all(test_str.as_bytes());
        }
    }

}

use std::collections::HashMap;

macro_rules! op_alias {
    ($ ($x:tt = $y:tt),* ) => {
        {
            let mut tmp_map = HashMap::new();
            $(
                tmp_map.insert($x, $y);
            )*
            tmp_map
        }
    }
}

lazy_static! {
    static ref OP_ALIAS: HashMap<&'static str, &'static str> = {
        // let mut m = HashMap::new();

        // m.insert("cmovae", "cmovnc");

        let mut m = op_alias!(
            "cmovae" = "cmovnc",
            "cmovb" = "cmovc",
            "cmovbe" = "cmovna",
            "cmove" = "cmovz",
            "cmovge" = "cmovnl",
            "cmovle" = "cmovng",
            "cmovnae" = "cmovc",
            "cmovnb" = "cmovnc",
            "cmovnbe" = "cmova",
            "cmovnge" = "cmovl",
            "cmovne" = "cmovnz",
            "cmovnle" = "cmovg",
            "cmovnp" = "cmovpo",
            "cmovp" = "cmovpe",

            "sal" = "shl",
            "sete" = "setz",
            "setae" = "setnc",
            "setbe" = "setna",
            "setge" = "setnl",
            "setnae" = "setc",
            "setle" = "setng",
            "setnb" = "setnc",
            "setnge" = "setl",
            "setnp" = "setpo",
            "setne" = "setnz",
            "setp" = "setpe",

            "jae" = "jnc",
            "jb" = "jc",
            "jbe" = "jna",
            "je" = "jz",
            "jge" = "jnl",
            "jle" = "jng",
            "jnae" = "jc",
            "jnb" = "jnc",
            "jnbe" = "ja",
            "jne" = "jnz",
            "jnle" = "jg",
            "jnge" = "jl",
            "jnp" = "jpo",
            "jp" = "jpe",

            "loopz" = "loope",
            "loopnz" = "loopne",

            "fwait" = "wait"
        );

        m
    };
}
use std::collections::HashSet;



fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tests.rs");
    let mut f = File::create(&dest_path).unwrap();
    let op_specs = testx64::opspecs_from_opmap();

    let mut processed = HashSet::new();


    for spec in op_specs {

        match spec.mnemonic {
            "monitorx" => continue,
            "mwaitx" => continue,
            _ => (),
        }

        let mut sp = spec.clone();

        match OP_ALIAS.get(spec.mnemonic) {
            Some(x) => sp.mnemonic = x,
            None => (),
        }

        let mnem = sp.mnemonic;

        if !processed.contains(sp.mnemonic) {
            testx64::generate_test(sp, &mut f);
        }

        processed.insert(mnem);
    }
}
