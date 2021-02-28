use dynamite::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize dynamite
    let mut dynamite = Dynamite::new();

    // Load langauge adapter
    dynamite.load_dynamic_library_language_adapter("./target/debug/libdynamite_python.so")?;

    // Print discovered api
    dbg!(dynamite.get_full_api());

    // Call a function provided by the language adapter ( just assuming for this example that we
    // know ahead of time that this function exists, it would error if it didn't ).
    let arg1 = &42f32;
    dynamite.call_function(
        &"python::test_function".to_string(),
        &[arg1 as *const f32 as *const Void],
    )?;

    Ok(())
}
