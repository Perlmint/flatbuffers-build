extern crate flatbuffers_run;

fn main() {
    let out_dir = std::env::current_dir().unwrap();
    if out_dir.join("reflection_generated.rs").exists() {
        return;
    }

    if let Ok(src_path) = std::env::var("FBS_REFLECTION_SRC") {
        let mut runner = flatbuffers_run::Runner::new();
        runner
            .rust(true)
            .add_definition(&src_path)
            .out_dir(&out_dir);
        runner.compile().expect("Failed to compile fbs reflection");
    } else {
        panic!("Faild to acquire reflection source. Specify this with FBS_REFLECTION_SRC env var");
    }
}
