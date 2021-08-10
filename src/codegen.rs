// Get address of native functions
macro_rules! libc_func {
    ($address:expr, $t:ty) => {
        std::mem::transmute::<*const usize, $t>($address as *const usize)
    };
}

// Jit calling convention
// r14 - program counter
// r15 - memory
pub struct Codegen {
    pub code: String,
    print_fn: extern "C" fn(u8) -> (),
    pc: &'static str,
    memory: &'static str,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            code: "[bits 64]\n".into(),
            print_fn: unsafe { libc_func!(libc::putchar, extern "C" fn(u8) -> ()) },
            pc: "r14",
            memory: "r15",
        }
    }

    pub fn alloc_rwx(size: usize) -> &'static mut [u8] {
        extern "C" {
            fn mmap(
                addr: *mut u8,
                length: usize,
                prot: i32,
                flags: i32,
                fd: i32,
                offset: usize,
            ) -> *mut u8;
        }

        unsafe {
            let ret = mmap(0 as *mut u8, size, 7, 34, -1, 0);
            assert!(!ret.is_null());
            std::slice::from_raw_parts_mut(ret, size)
        }
    }

    pub fn setup(&mut self) {
        self.code += &format!("setup:\n");
        self.code += &format!("push {pc}\n", pc = self.pc);
        self.code += &format!("push {memory}\n", memory = self.memory);
        self.code += &format!("xor {pc}, {pc}\n", pc = self.pc);
        self.code += &format!("mov {memory}, rdi\n", memory = self.memory);
    }

    pub fn cleanup(&mut self) {
        self.code += &format!("cleanup:\n");
        self.code += &format!("pop {memory}\n", memory = self.memory);
        self.code += &format!("pop {pc}\n", pc = self.pc);
        self.code += &format!("xor rax, rax\n");
        self.code += &format!("ret");
    }

    pub fn next(&mut self, value: usize) {
        self.code += &format!("add {pc}, {value}\n", pc = self.pc, value = value);
    }

    pub fn prev(&mut self, value: usize) {
        self.code += &format!("sub {pc}, {value}\n", pc = self.pc, value = value);
    }

    pub fn inc(&mut self, value: usize) {
        self.code += &format!(
            "add byte [{memory} + {pc}], {value}\n",
            memory = self.memory,
            pc = self.pc,
            value = value
        );
    }

    pub fn dec(&mut self, value: usize) {
        self.code += &format!(
            "sub byte [{memory} + {pc}], {value}\n",
            memory = self.memory,
            pc = self.pc,
            value = value
        );
    }

    pub fn jumpf(&mut self, offset: usize) {
        self.code += &format!(".start_{}:\n", offset);
        self.code += &format!(
            "cmp byte [{memory} + {pc}], 0x0\n",
            memory = self.memory,
            pc = self.pc
        );
        self.code += &format!("je .end_{}\n", offset);
    }

    pub fn jumpb(&mut self, offset: usize) {
        self.code += &format!(
            "cmp byte [{memory} + {pc}], 0x0\n",
            memory = self.memory,
            pc = self.pc
        );
        self.code += &format!("jne .start_{}\n", offset);
        self.code += &format!(".end_{}:\n", offset);
    }

    pub fn print(&mut self) {
        self.code += &format!(
            "movzx edi, byte [{memory} + {pc}]\n",
            memory = self.memory,
            pc = self.pc
        );
        self.code += &format!("mov rax, {:?}\n", self.print_fn);
        self.code += &format!("call rax\n");
    }
}
