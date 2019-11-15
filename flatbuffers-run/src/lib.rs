extern crate bitflags;
use bitflags::bitflags;
use std::ffi::OsStr;
use std::path::PathBuf;

const FLATC_PATH: &'static str = env!("FLATC");

bitflags! {
    pub struct CompileFlags: u32 {
        const RUST =   0b00000001;
        const SCHEMA = 0b00000010;
    }
}

pub struct Runner {
    flags: CompileFlags,
    definitions: Vec<PathBuf>,
    includes: Vec<PathBuf>,
    out_dir: Option<PathBuf>,
}

pub struct CompileResult {
    pub rust: Option<PathBuf>,
    pub schema: Option<PathBuf>,
}

impl Runner {
    pub fn new() -> Self {
        Runner {
            flags: CompileFlags::empty(),
            definitions: Default::default(),
            includes: Default::default(),
            out_dir: None,
        }
    }

    pub fn get_definitions(&self) -> &[PathBuf] {
        self.definitions.as_slice()
    }

    pub fn add_definition<T: ?Sized + AsRef<OsStr>>(&mut self, definition: &T) -> &mut Self {
        self.definitions.push(PathBuf::from(definition));

        self
    }

    pub fn add_definitions<T: ?Sized + AsRef<OsStr>>(&mut self, definitions: &[&T]) -> &mut Self {
        self.definitions
            .extend(definitions.iter().map(|s| PathBuf::from(*s)));

        self
    }

    pub fn add_include<T: ?Sized + AsRef<OsStr>>(&mut self, include: &T) -> &mut Self {
        self.includes.push(PathBuf::from(include));

        self
    }

    pub fn add_includes<T: ?Sized + AsRef<OsStr>>(&mut self, includes: &[&T]) -> &mut Self {
        self.includes
            .extend(includes.iter().map(|s| PathBuf::from(*s)));

        self
    }

    pub fn out_dir<T: ?Sized + AsRef<OsStr>>(&mut self, dir: &T) -> &mut Self {
        self.out_dir = Some(PathBuf::from(dir));

        self
    }

    pub fn rust(&mut self, enable: bool) -> &mut Self {
        self.flags.set(CompileFlags::RUST, enable);
        self
    }

    pub fn schema(&mut self, enable: bool) -> &mut Self {
        self.flags.set(CompileFlags::SCHEMA, enable);
        self
    }

    pub fn compile(self) -> std::io::Result<Vec<CompileResult>> {
        use std::process::Command;

        let mut command = Command::new(FLATC_PATH);
        let gen_rust = self.flags.contains(CompileFlags::RUST);
        if gen_rust {
            command.arg("--rust");
        }
        let gen_schema = self.flags.contains(CompileFlags::SCHEMA);
        if gen_schema {
            command.args(&["--schema", "--binary", "--bfbs-comments", "--bfbs-builtins"]);
        }
        let out_dir = if let Some(out_dir) = self.out_dir {
            command.arg("-o");
            command.arg(&out_dir);
            out_dir
        } else {
            std::env::current_dir().unwrap().to_owned()
        };

        for definition in &self.definitions {
            command.arg(definition);
        }

        let mut child = command.spawn()?;
        child.wait()?;

        Ok(self
            .definitions
            .iter()
            .map(|def| {
                let file_name = std::path::Path::new(&def).file_stem().unwrap();
                let file_name = file_name.to_str().unwrap();
                CompileResult {
                    rust: if gen_rust {
                        Some(out_dir.join(format!("{}_generated.rs", file_name)))
                    } else {
                        None
                    },
                    schema: if gen_schema {
                        Some(out_dir.join(format!("{}.bfbs", file_name)))
                    } else {
                        None
                    },
                }
            })
            .collect())
    }
}
