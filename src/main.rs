use std::fs::{self, DirEntry, File, OpenOptions};
use std::path::Path;
use std::process::Command;
use std::io::{Read, Write, BufReader};

fn main() {

    // 获取当前目录
    let args: Vec<String> = std::env::args().collect();
    let path = Path::new(&args[0]);
    let path = path.parent().unwrap();
    let current_dir = path.to_str().unwrap().trim_end_matches("/");

    // 获取home目录
    let home_dir = dirs::home_dir().unwrap();
    let home_dir = home_dir.to_str().unwrap().trim_end_matches("/");

    // 获取使用的是哪种shell
    let output = Command::new("sh")
        .arg("-c")
        .arg("echo $SHELL")
        .output()
        .expect("failed to execute process");

    let shell_name = String::from_utf8(output.stdout).unwrap();
    let shell_name = shell_name.trim().trim_start_matches("/").trim_start_matches("/");
    let mut profile_name = "".to_string();
    let shell_name_info: Vec<&str> = shell_name.split("/").collect();
    if let Some(shell_name) = shell_name_info.last() {
        profile_name = format!(".{}rc", shell_name);
    }

    let profile_filepath_string = format!("{}/{}", home_dir, profile_name);
    let profile_filepath = Path::new(&profile_filepath_string);

    // 判断是否已有安装目录
    let panda_dir_string = format!("{}/.panda_api/", home_dir);
    let panda_dir = Path::new(&panda_dir_string);
    if !panda_dir.exists() {
        // 如果文件夹不存在，创建文件夹
        match std::fs::create_dir_all(&panda_dir_string) {
            Ok(_) => (),
            Err(e) => {
                println!("create folder failed {} {:?}", &panda_dir_string, e);
            }
        }
    }

    let install_files = ["panda", "theme"];
    for file in &install_files {
        // 复制命令到文件夹下
        let cp_command = format!("cp -rf {}/{} {}", current_dir, file, &panda_dir_string);
        if cfg!(target_os = "windows") {
            // 执行 windows 下 文件复制
        } else {
            // 执行 Linux 下文件复制
            let r = Command::new("sh")
                .arg("-c")
                .arg(&cp_command)
                .output()
                .expect(&format!("failed to cp file {} to {}", file, &panda_dir_string));
        };
    }

    let profile_content = r#"export PATH="$HOME/.panda_api:$PATH""#;
    // 编辑profile文件
    let mut has_profile_content = false;

    if profile_filepath.exists() {
        // 读取里面的内容，看是否已经有，没有的话就要编辑加上内容
        let mut content = fs::read_to_string(&profile_filepath_string).expect(&format!("failed to read file {}", &profile_filepath_string));
        if content.contains(profile_content) {
            has_profile_content = true;
        }
    }

    if !has_profile_content {
        let mut file_options = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(&profile_filepath_string);

        match file_options {
            Ok(mut file) => {
                let new_content = format!("{}\n", profile_content);
                file.write_all(new_content.as_bytes()).expect(&format!("failed to write data to file {}", &profile_filepath_string));
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    let profile_list = [".zshrc", ".bashrc", ".cshrc"];

    for profile_file in &profile_list {
        let profile_filepath_string = format!("{}/{}", home_dir, profile_file);
        let profile_filepath = Path::new(&profile_filepath_string);

        if profile_filepath.exists() {
            // 读取里面的内容，看是否已经有，没有的话就要编辑加上内容
            let mut content = fs::read_to_string(&profile_filepath_string).expect(&format!("failed to read file {}", &profile_filepath_string));
            if content.contains(profile_content) {
                continue;
            } else {
                let mut file_options = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .append(true)
                    .open(&profile_filepath_string);

                match file_options {
                    Ok(mut file) => {
                        let new_content = format!("{}\n", profile_content);
                        file.write_all(new_content.as_bytes()).expect(&format!("failed to write data to file {}", &profile_filepath_string));
                    }
                    Err(e) => {
                        println!("不存在 {} {:?}", profile_filepath_string, e);
                        continue;
                    }
                }
            }
        }
    }


    println!("Congratulations!\nPanda api install done!\nYou can run pana in your api docs folder now.");
}
