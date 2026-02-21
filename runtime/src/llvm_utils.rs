// use inkwell::{context::Context, types::IntType};

// #[allow(dead_code)]
// pub fn usize_type<'a>(ctx: &'a Context) -> IntType<'a> {
//     cfg_select! {
//         target_pointer_width = "16" => { ctx.i16_type() }
//         target_pointer_width = "32" => { ctx.i32_type() }
//         target_pointer_width = "64" => { ctx.i64_type() }
//     }
// }
