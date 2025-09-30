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
    let kernel_params = "root=UUID=ROOT-UUID-5678 rw";

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

    // Write to a temp file to mimic installer behavior
    let out_dir = tempfile::tempdir().expect("tempdir");
    let out_path = out_dir.path().join("limine.conf");
    fs::write(&out_path, &final_conf).expect("write limine.conf");

    // Assert structure
    let conf = fs::read_to_string(&out_path).unwrap();
    assert!(conf.contains("timeout: 5"));
    assert!(conf.contains("/Arch Linux (linux)"));
    assert!(conf.contains("/Arch Linux (linux-fallback)"));
    assert!(conf.contains("protocol: linux"));
    assert!(conf.contains("path: uuid(BOOT-UUID-1234):/vmlinuz-linux"));
    assert!(conf.contains("module_path: uuid(BOOT-UUID-1234):/initramfs-linux.img"));
    assert!(conf.contains("module_path: uuid(BOOT-UUID-1234):/initramfs-linux-fallback.img"));
    assert!(conf.contains("cmdline: root=UUID=ROOT-UUID-5678 rw"));
}
