use crate::core::{addResource, exportResource, ResourceHead, getResourceList, CompressMode};
use std::path::PathBuf;
use std::fs;
use std::any::Any;

lazy_static! {
    static ref ID: String = "1".to_string();
    // static ref BASE_PATH: PathBuf = PathBuf::from(r"D:\Project\back-end\Rust\Appender\test");
    static ref BASE_PATH: PathBuf = PathBuf::from(r"C:\Users\Administrator\Desktop");
    static ref TARGET_FILE: PathBuf = BASE_PATH.join("Notepad.exe");
    static ref SOURCE_FILE: PathBuf = BASE_PATH.join("课堂资料day01_21.zip");
    // static ref SOURCE_FILE2: PathBuf = BASE_PATH.join("Roberto_Cacciapaglia.pdf");
    static ref OUTPUT_PATH: PathBuf = BASE_PATH.join("输出.zip");
}

/// 增加资源测试
#[test]
fn addResourceTest() {
    addResource((&*TARGET_FILE).as_ref(), (&*SOURCE_FILE).as_ref(), &ID, None, None).unwrap();
    println!("资源增加成功");
}

/// 释放资源测试
#[test]
fn exportResourceTest() {
    // exportResource((&*TARGET_FILE).as_ref(), &ID, OUTPUT_PATH.as_ref()).unwrap();
    exportResource(&*PathBuf::from(r"C:\Users\Administrator\Desktop\notepad.exe"), "1", &*PathBuf::from(r"C:\Users\Administrator\Desktop")).unwrap();
    println!("资源释放成功");
}

/// 自动测试
#[test]
fn autoTest() {
    let testPath = TARGET_FILE.parent().unwrap().join("test");

    fs::remove_dir_all(&testPath);
    fs::create_dir(&testPath);

    let testTargetFile = testPath.join(TARGET_FILE.file_name().unwrap());
    let testOutputFile = testPath.join(OUTPUT_PATH.file_name().unwrap());
    fs::copy(TARGET_FILE.clone(), &testTargetFile);

    addResource((&*testTargetFile).as_ref(), (&*SOURCE_FILE).as_ref(), &ID, None, None).unwrap();
    exportResource((&*testTargetFile).as_ref(), &ID, &*testPath).unwrap();
    println!("测试成功");
}

/// 实例测试
#[test]
fn Test() {
    // let defaultResourceHead = ResourceHead::default();

    // let resource = ResourceHead::new("1", 123, "aaa.exe",CompressMode::None);
    let resource = ResourceHead::new("1", 39278156, 100,"day01_21.zip", CompressMode::None);
    // println!("{:?}", resource);

    let bin = &resource.to_bytes().unwrap();
    // println!("{:?}", bin);

    // Length
    // println!("{:?}", bin.len());
    // println!("{}", defaultResourceHead.getLen());

    // println!("{:?}", format!("{:^length$}", "abcd", length = 255).chars().count());
    // println!("{:?}", format!("{:^length$}", "我我我我w", length = 255).chars().count());
    println!("{:?}", format!("{:^length$}", "abcd", length = 5).as_bytes());
    let str = "abcd";
    println!("{:?}", format!("{:^length$}", str, length = 5 - str.len() + str.chars().count()).as_bytes());
}

use serde::{Serialize, Deserialize};
use crate::util::{decompressFile, compressionFile};

#[test]
fn test2() {
    // println!("{:?}", getResourceList(&*PathBuf::from(r"C:\Users\Administrator.W10-20201229857\Desktop\新建文件夹\test\Notepad.exe")));

    let aa = Astruct {
        Name: "".to_string(),
        CompressMode: CompressMode::Compress,
    };
    println!("{:?}", bincode::serialize(&aa));
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Astruct {
    Name: String,
    CompressMode: CompressMode,
}

#[test]
fn test3() {
    compressionFile(&*PathBuf::from(r"D:\Project\back-end\Rust\Appender\test\hh.exe"), &*PathBuf::from(r"D:\Project\back-end\Rust\Appender\test\temp"), 5);
    // decompressFile(&*PathBuf::from(r"D:\Project\back-end\Rust\Appender\test\temp"), &*PathBuf::from(r"D:\Project\back-end\Rust\Appender\test\temp.exe"));
}
