use std::env;
use std::fs::{read_dir, File};
use std::io::{Result, Write};

fn main() {
	let target=env::var("TARGET").unwrap();
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed=../user/target/{}", target);
    insert_app_data().unwrap();
}

fn insert_app_data() -> Result<()> {
	let target=env::var("TARGET").unwrap();
    let mut f = File::create("src/link_app.S").unwrap();
    let mut apps: Vec<_> = read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    writeln!(
        f,
        r#"
    .align 8
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "../user/target/{2}/release/{1}.bin"
app_{0}_end:"#,
            idx, app, target
        )?;
    }
    Ok(())
}
