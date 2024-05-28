use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::env;

fn get_target_path() -> Option<String> {
    // 获取环境变量 "TARGET" 的值
    if let Ok(target) = env::var("TARGET") {
        let target_path = format!("../user/target/{}/release/", target);
        Some(target_path)
    } else {
        None
    }
}

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    if let Some(target_path) = get_target_path() {
        println!("cargo:rerun-if-changed={}", target_path);
    }
    else{
        println!("环境变量 TARGET 未设置");
        return;
    }
    insert_app_data().unwrap();
}


fn insert_app_data() -> Result<()> {
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
    .align 3
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
        if let Some(target_path) = get_target_path() {
            writeln!(
                f,
                r#"
        .section .data
        .global app_{0}_start
        .global app_{0}_end
        .align 3
    app_{0}_start:
        .incbin "{2}{1}"
    app_{0}_end:"#,
                idx, app, target_path
            )?;
        }
    }
    Ok(())
}
