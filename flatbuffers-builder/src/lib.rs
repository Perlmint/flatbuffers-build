#[path = "../reflection_generated.rs"]
pub mod fbs_schema;
use std::ffi::OsStr;

pub trait ServiceGenerator {
    fn write_service<'a>(
        &mut self,
        writer: &mut dyn std::io::Write,
        schema: fbs_schema::reflection::Schema<'a>,
    ) -> std::io::Result<()>;
}

pub struct Builder {
    runner: flatbuffers_run::Runner,
    generator: Option<Box<dyn ServiceGenerator>>,
}

impl Builder {
    pub fn new<T: ?Sized + AsRef<OsStr>>(out_dir: &T) -> Self {
        Builder {
            runner: {
                let mut runner = flatbuffers_run::Runner::new();
                runner.out_dir(out_dir);

                runner
            },
            generator: None,
        }
    }

    pub fn add_definition<T: ?Sized + AsRef<OsStr>>(&mut self, definition: &T) -> &mut Self {
        self.runner.add_definition(definition);

        self
    }

    pub fn add_definitions<T: ?Sized + AsRef<OsStr>>(&mut self, definitions: &[&T]) -> &mut Self {
        self.runner.add_definitions(definitions);

        self
    }

    pub fn add_include<T: ?Sized + AsRef<OsStr>>(&mut self, include: &T) -> &mut Self {
        self.runner.add_include(include);

        self
    }

    pub fn add_includes<T: ?Sized + AsRef<OsStr>>(&mut self, includes: &[&T]) -> &mut Self {
        self.runner.add_includes(includes);

        self
    }

    pub fn generator(&mut self, generator: Box<dyn ServiceGenerator>) -> &mut Self {
        self.generator = Some(generator);

        self
    }

    pub fn generate(mut self) -> std::io::Result<()> {
        self.runner.rust(true);

        if self.generator.is_some() {
            self.runner.schema(true);
        }

        for def in self.runner.get_definitions() {
            println!("cargo:rerun-if-changed={}", def.to_str().unwrap());
        }
        let generateds = self.runner.compile()?;
        if self.generator.is_none() {
            return Ok(());
        }

        let mut generator = self.generator.unwrap();
        let mut schema_buffer: Vec<u8> = Default::default();
        for item in generateds {
            use std::fs::{File, OpenOptions};
            use std::io::{Read, Write};

            let schema_path = item.schema.unwrap();
            let schema = {
                let mut schema = File::open(schema_path.clone())?;
                schema.read_to_end(&mut schema_buffer)?;

                fbs_schema::reflection::get_root_as_schema(&schema_buffer)
            };

            if schema.services().is_none() {
                continue;
            }

            let mut src = {
                let src_path = item.rust.unwrap();
                let mut src = OpenOptions::new();

                src.append(true).open(src_path)?
            };

            writeln!(src)?;

            generator.write_service(&mut src, schema)?;
            src.flush()?;

            drop(src);
            std::fs::remove_file(schema_path)?;
        }

        Ok(())
    }
}
