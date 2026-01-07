#![allow(clippy::derivable_impls)]

pub mod runtime {
    use crate::path_searcher;
    use bon::Builder;
    use getset::{CopyGetters, Getters, MutGetters};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    #[derive(
        Clone, Getters, MutGetters, derive_more::Debug, Builder, CopyGetters, Deserialize, Serialize,
    )]
    #[getset(get = "pub")]
    #[allow(clippy::type_complexity)]
    pub struct VMConfig {
        #[builder(default)]
        default_cpu_config: CPUConfig,
        #[cfg_attr(debug_assertions, builder(default = true))]
        #[cfg_attr(not(debug_assertions), builder(default = false))]
        #[getset(skip)]
        #[get_copy = "pub"]
        is_dynamic_checking_enabled: bool,
        #[debug(skip)]
        #[getset(get_mut = "pub")]
        #[serde(skip)]
        assembly_lookuper: Option<Arc<dyn Fn(&str) -> Option<String>>>,
    }

    impl Default for VMConfig {
        fn default() -> Self {
            Self {
                default_cpu_config: CPUConfig::default(),
                #[cfg(debug_assertions)]
                is_dynamic_checking_enabled: true,
                #[cfg(not(debug_assertions))]
                is_dynamic_checking_enabled: false,
                assembly_lookuper: Some(Arc::new(|name| {
                    let stdlib = path_searcher::get_stdlib_dir().ok()?;
                    let std_dir = std::fs::read_dir(&stdlib).ok()?;
                    for entry in std_dir {
                        let entry = entry.ok()?;
                        if entry.file_type().unwrap().is_file() && *entry.file_name() == *name {
                            return Some(entry.path().to_str()?.to_owned());
                        }
                    }
                    None
                })),
            }
        }
    }

    #[derive(Clone, Debug, Getters, CopyGetters, Deserialize, Serialize)]
    pub struct CPUConfig {
        #[get_copy = "pub"]
        default_register_num: u64,
    }

    impl Default for CPUConfig {
        fn default() -> Self {
            Self {
                default_register_num: u8::MAX as _,
            }
        }
    }
}

pub mod compiler {
    use bon::Builder;
    use getset::Getters;
    use serde::{Deserialize, Serialize};

    #[derive(Getters, Builder, Clone, Debug, Deserialize, Serialize)]
    #[getset(get = "pub")]
    pub struct CompilerConfig {}

    impl Default for CompilerConfig {
        fn default() -> Self {
            Self {}
        }
    }

    #[derive(Getters, Builder, Clone, Debug, Deserialize, Serialize)]
    #[getset(get = "pub")]
    pub struct CompileServiceConfig {
        default_compiler_config: CompilerConfig,
        stdlib_dir: String,
        verbose: bool,
    }

    impl Default for CompileServiceConfig {
        fn default() -> Self {
            Self {
                default_compiler_config: Default::default(),
                stdlib_dir: crate::path_searcher::get_stdlib_dir().unwrap(),
                verbose: cfg!(debug_assertions),
            }
        }
    }

    #[derive(Clone, Copy, Default, Debug)]
    pub enum OptimizationLevel {
        #[cfg_attr(debug_assertions, default)]
        Debug,
        #[cfg_attr(not(debug_assertions), default)]
        Release,
    }

    impl OptimizationLevel {
        pub const fn as_str(&self) -> &'static str {
            match self {
                OptimizationLevel::Debug => "debug",
                OptimizationLevel::Release => "release",
            }
        }
    }
}
