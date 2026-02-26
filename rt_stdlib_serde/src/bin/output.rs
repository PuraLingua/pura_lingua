use std::fs::File;

enum OutputKind {
    Json,
}

fn main() -> global::Result<()> {
    let mut file = cfg_select! {
        windows => { unsafe { <File as std::os::windows::io::FromRawHandle>::from_raw_handle(std::io::stdout()) } }
        unix => { File::create("/dev/stdout")? }
    };
    let mut kind = OutputKind::Json;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" => {
                let path = args.next().unwrap();
                file = File::create(path)?;
            }
            "--kind" => match args.next().unwrap().as_str() {
                "json" => {
                    kind = OutputKind::Json;
                }
                x => panic!("Unknown kind {x}"),
            },
            _ => panic!("Unkown arg {arg}"),
        }
    }
    match kind {
        OutputKind::Json => {
            let information = pura_lingua_runtime_stdlib_serde::get_all_core_type_info();
            let mut serializer = serde_json::Serializer::with_formatter(
                file,
                serde_json::ser::PrettyFormatter::with_indent(b"    "),
            );
            <_ as serde::Serialize>::serialize(&information, &mut serializer)?;
        }
    }
    Ok(())
}
