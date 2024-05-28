use std::fs;

use serde::{Deserialize, Serialize};
use toml::Table;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchPackage {
    name: String,
    git: String,
    rev: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoPackage {
    name: String,
    source: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoLock {
    package: Vec<CargoPackage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitPatch {
    git: String,
    commit: String,
}

fn get_patch_table(cargo_path: &String) -> anyhow::Result<Vec<CargoPackage>> {
    let cargo_lock: CargoLock =
        toml::from_str(&fs::read_to_string(format!("{}/Cargo.lock", cargo_path)).unwrap()).unwrap();
    let patch_table = cargo_lock
        .package
        .into_iter()
        .filter(|x| match x.source {
            Some(ref source) => source.starts_with("git+https://github.com"),
            None => false,
        })
        .collect();
    Ok(patch_table)
}

pub fn do_patch(
    cargo_path: &String,
    patch_name: &String,
    patch_repo_name: &String,
    commit: &String,
) -> anyhow::Result<()> {
    // 提取 patch_repo_name 的 repository 名称，并在前面加一个 /
    // 防止指向同一个仓库，导致报错`points to the same source, but patches must point to different sources`
    let names = patch_repo_name.split('/').collect::<Vec<&str>>();
    if names.len() != 2 {
        return Err(anyhow!("This is not a valid patch repo name"));
    }
    let patch_git = format!("https://github.com/{}//{}.git", names[0], names[1]);
    // Check if the patch name is available
    let patch_table = get_patch_table(cargo_path)?;
    let patch = patch_table
        .iter()
        .find(|x| x.name == *patch_name)
        .ok_or(anyhow!("Can't find matched patch name"))?;

    // Get the patch info from the specific package
    let urls: Vec<&str> = patch.source.as_ref().unwrap().split("#").collect();
    if urls.len() < 2 {
        return Err(anyhow!("This is not a valid patch source"));
    }
    let git_url_end = urls[0].find('?').unwrap_or(urls[0].len());
    let git_url = &urls[0][4..git_url_end];

    let mut cargo_toml: Table =
        toml::from_str(&fs::read_to_string(format!("{}/Cargo.toml", cargo_path))?)?;
    if !cargo_toml.contains_key("patch") {
        cargo_toml.insert(String::from("patch"), toml::Value::Table(Table::new()));
    }
    let patch_table = cargo_toml.get_mut("patch").unwrap().as_table_mut().unwrap();

    if !patch_table.contains_key(git_url) {
        patch_table.insert(git_url.to_string(), toml::Value::Table(Table::new()));
    }
    let crates_io_table = patch_table
        .get_mut(git_url)
        .unwrap()
        .as_table_mut()
        .unwrap();

    let mut git_table = toml::value::Table::new();
    git_table.insert("git".to_string(), toml::Value::String(patch_git.clone()));
    git_table.insert("rev".to_string(), toml::Value::String(commit.clone()));

    crates_io_table.insert(patch_name.to_string(), toml::Value::Table(git_table));
    fs::write(
        format!("{}/Cargo.toml", cargo_path),
        toml::to_string(&cargo_toml).unwrap(),
    )
    .unwrap();
    Ok(())
}
