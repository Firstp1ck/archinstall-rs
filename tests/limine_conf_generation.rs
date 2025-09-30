use std::fs;

// A very lightweight unit-style test to validate our limine.conf content generator.
// This test simulates producing the KERNELS_SECTION and injecting it into an example template
// and then checks that placeholders are resolved and expected lines exist.
#[test]
fn generates_limine_conf_from_template() {
    // Load the example template
    let tmpl_path = std::path::Path::new("assets/limine/limine.conf.example");
    let template = fs::read_to_string(&tmpl_path).expect("missing limine.conf.example template");

    // Simulated values that the shell script would expand
    let path_root = "uuid(BOOT-UUID-1234)";
    // Use a realistic UUID to validate replacement of manual placeholder line
    let uuid = "123e4567-e89b-12d3-a456-426614174000";
    let kernel_params = format!("root=UUID={} rw", uuid);

    // Simulate selected kernels: linux + fallback
    let kernels = vec!["linux"]; // only one for this unit test

    // Build the generated section the same way as our script would emit, but directly in Rust
    let mut generated = String::new();
    for k in &kernels {
        for variant in ["", "-fallback"] {
            generated.push_str(&format!(
                "\n/Arch Linux ({}{})\n    protocol: linux\n    path: {}:/vmlinuz-{}\n    cmdline: {}\n    module_path: {}:/initramfs-{}{}.img\n",
                k, variant, path_root, k, kernel_params, path_root, k, variant
            ));
        }
    }

    // Inject into template
    let final_conf = template.replace("{{KERNELS_SECTION}}", &generated);

    // Write directly into assets/limine as requested
    let out_path = std::path::Path::new("assets/limine/limine.conf");
    fs::create_dir_all(out_path.parent().unwrap()).expect("create assets/limine");
    fs::write(&out_path, &final_conf).expect("write assets/limine/limine.conf");

    // Assert structure
    let conf = fs::read_to_string(&out_path).unwrap();
    assert!(conf.contains("timeout: 5"));
    assert!(conf.contains("/Arch Linux (linux)"));
    assert!(conf.contains("/Arch Linux (linux-fallback)"));
    assert!(conf.contains("protocol: linux"));
    assert!(conf.contains("path: uuid(BOOT-UUID-1234):/vmlinuz-linux"));
    assert!(conf.contains("module_path: uuid(BOOT-UUID-1234):/initramfs-linux.img"));
    assert!(conf.contains("module_path: uuid(BOOT-UUID-1234):/initramfs-linux-fallback.img"));

    // Verify that the cmdline contains our UUID and not the manual placeholder
    let expected_line = format!("cmdline: root=UUID={} rw", uuid);
    assert!(conf.contains(&expected_line), "cmdline line missing: {}\nconf:\n{}", expected_line, conf);
    assert!(
        !conf.contains("cmdline: root=UUID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"),
        "manual placeholder should not be present in generated config"
    );
}
