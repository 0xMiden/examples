use miden_stdlib::StdLibrary;
use miden_vm::{Assembler, Kernel, Program, ProgramInfo};

pub struct MidenProgram {
    pub assembler: Assembler,
    pub masm_code: String,
    pub program: Option<Program>,
    pub program_info: Option<ProgramInfo>,
}

impl MidenProgram {
    pub fn new<S: AsRef<str>>(code_as_str: S) -> Self {
        Self {
            assembler: Assembler::default(),
            masm_code: code_as_str.as_ref().to_string(),
            program: None,
            program_info: None,
        }
    }

    pub fn compile_program(&mut self) -> Result<(), String> {
        self.assembler = Assembler::default()
            .with_static_library(StdLibrary::default())
            .map_err(|err| err.to_string())?
            .with_debug_mode(false);

        self.program = Some(
            self.assembler
                .clone()
                .assemble_program(&self.masm_code)
                .map_err(|err| format!("Failed to compile program - {err}"))?,
        );

        self.program_info = Some(ProgramInfo::new(
            self.program
                .clone()
                .expect("Could not compile program")
                .hash(),
            Kernel::default(),
        ));

        Ok(())
    }
}
