#![feature(iterator_try_collect)]

use std::path::PathBuf;

use clap::Parser;
use pura_lingua::runtime::virtual_machine::CpuID;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    assemblies: Vec<String>,
    /// Format: <assembly name>/<class name>[/<method name>(`Main` by default)]
    ///
    /// <class name> could be `.`, which means that class has the same name as the assembly
    #[arg(long, value_parser = parse_main)]
    main: (String, String, String),
    args: Vec<String>,
}

fn search_assembly(name: &str) -> pura_lingua::global::Result<PathBuf> {
    if name.contains(std::path::MAIN_SEPARATOR) {
        return Ok(PathBuf::from(name));
    }
    let name_in_fs = name.replace("::", "_").replace('`', "_");
    let mut prebuilt_path =
        PathBuf::from(pura_lingua::global::path_searcher::get_stdlib_prebuilt_dir()?);
    prebuilt_path.push(&name_in_fs);
    if prebuilt_path.exists() {
        return Ok(prebuilt_path);
    }
    let mut cwd_path = std::env::current_dir()?;
    cwd_path.push(&name_in_fs);
    if cwd_path.exists() {
        Ok(cwd_path)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Assembly not found").into())
    }
}

fn parse_main(x: &str) -> pura_lingua::global::Result<(String, String, String)> {
    // cSpell:disable-next-line
    if let Some((rest, main)) = x.rsplit_once('/')
        && let Some((assembly, class)) = rest.split_once('/')
    {
        Ok((
            assembly.to_owned(),
            if class == "." {
                assembly.to_owned()
            } else {
                class.to_owned()
            },
            main.to_owned(),
        ))
    } else {
        let (assembly, class) = x
            .split_once('/')
            .ok_or(pura_lingua::global::errors::anyhow!("No class found"))?;
        Ok((assembly.to_owned(), class.to_owned(), "Main".to_owned()))
    }
}

fn main() -> pura_lingua::global::Result<()> {
    let cli = Cli::parse();
    let vm = pura_lingua::runtime::virtual_machine::global_vm();

    let binaries = cli
        .assemblies
        .iter()
        .map(
            |x: &String| -> pura_lingua::global::Result<pura_lingua::binary::prelude::Assembly> {
                let path = search_assembly(x)?;
                pura_lingua::binary::prelude::Assembly::from_path(path).map_err(From::from)
            },
        )
        .try_collect::<Vec<_>>()?;
    vm.assembly_manager().load_binaries(&binaries)?;

    let (main_assembly, main_class, main_method) = cli.main;
    let assembly = vm
        .assembly_manager()
        .get_assembly_by_name(&main_assembly)
        .unwrap()
        .unwrap();
    let class = assembly
        .find_class(&main_class)
        .unwrap()
        .expect(&format!("Class {main_class} not found"));
    let mt = unsafe { class.as_ref().method_table_ref() };
    let method_id = mt
        .find_last_method_by_name_ret_id(&main_method)
        .expect("Method not found");
    let main_method = mt.get_method(method_id).unwrap();

    let mut cpu = CpuID::new_write_global();
    unsafe {
        cpu.invoke_main_and_exit(main_method.as_ref(), cli.args);
    }
}
