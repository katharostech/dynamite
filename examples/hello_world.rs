use dynamite::*;

/// A Rust function we want to be able to call from scripting. To make this function callable from
/// scripting languages we can annotate it with the `#[stockpile_funciton]` macro, which creates
/// script bindings and adds it to the Dynamite stockpile.
///
/// The method will be accessible to scripts under the path `[module_name]::[function_name]`, or
/// specifically, in this case, `hello_world::rust_func`
#[stockpile_function]
fn rust_func(a: &i32, b: &i32) -> &'static i32 {
    println!("Hello from Rust!! Computing: {} + {}", a, b);

    let value = Box::new(a + b);
    let ptr = Box::leak(value);

    ptr
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize dynamite
    let mut dynamite = Dynamite::new();

    // Add our native Rust adapter
    dynamite.add_stockpile()?;

    // Load langauge adapter ( relatively safe, but still unsafe because dynamic libraries could do
    // _anything_ 👀 )
    unsafe {
        dynamite.load_dynamic_library_language_adapter("./target/debug/libdynamite_python.so")?
    };

    // Print discovered api
    dbg!(dynamite.get_full_api());

    // Call a function provided by the language adapter ( just assuming for this example that we
    // know ahead of time that this function exists, it would error if it didn't ). This is also
    // unsafe because your language adapter could mis-behave.
    let arg1 = &42f32;
    unsafe {
        dynamite.call_function(
            &"python::test_function".to_string(),
            &[arg1 as *const f32 as *const Void],
        );
    }

    Ok(())
}
