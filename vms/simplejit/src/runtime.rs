use {
    memmap2::MmapMut,
    std::{
        io::{Read, Write},
        mem,
    },
    util::BfResult,
};

type AsmEntryPoint = extern "C" fn();

const MEMORY_SIZE: usize = 30000;

pub struct Runtime<'a> {
    memory: [u8; MEMORY_SIZE],
    stdin: &'a mut dyn Read,
    stdout: &'a mut dyn Write,
}

impl<'a> Runtime<'a> {
    pub fn new(stdin: &'a mut dyn Read, stdout: &'a mut dyn Write) -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            stdin,
            stdout,
        }
    }

    pub fn memory_ptr(&mut self) -> *mut u8 {
        self.memory.as_mut_ptr()
    }

    pub fn run(&self, machine_code: &[u8]) -> BfResult<()> {
        let mut mmap = MmapMut::map_anon(machine_code.len())?;
        mmap.copy_from_slice(machine_code);

        let mmap = mmap.make_exec()?;
        let entry_point_pointer = mmap.as_ptr() as *const ();
        let entry_point =
            unsafe { mem::transmute::<*const (), AsmEntryPoint>(entry_point_pointer) };
        entry_point();

        Ok(())
    }

    pub extern "C" fn read(&mut self) -> u8 {
        let mut c = [0; 1];
        self.stdin.read_exact(&mut c).unwrap();

        c[0]
    }

    pub extern "C" fn write(&mut self, byte: u8) {
        self.stdout.write_all(&[byte]).unwrap();
        self.stdout.flush().unwrap();
    }
}
