extern crate embed_resource;
use embed_resource::CompilationResult::*;

fn main() {
    #[cfg(windows)]
    {
        let version = env!("CARGO_PKG_VERSION");
        // "0.1.0" -> "0,1,0,0"
        let version_commas = version.replace('.', ",") + ",0";

        let header = format!(
            "#define VERSION_INT  {}\n#define VERSION_STR  \"{}\"\n",
            version_commas, version
        );
        std::fs::write("version.h", header).expect("Failed to write version.h");

        match embed_resource::compile("res.rc", embed_resource::NONE) {
            NotAttempted(x) => {
                panic!("{}", x)
            }
            Failed(x) => {
                panic!("{}", x)
            }
            _ => {}
        };
    }
}
