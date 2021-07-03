use std::cmp::Ordering;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use crate::core::BUFFER_SIZE;
use std::error::Error;

/// 寻找字节
/// # 返回值
/// 返回找到的字节位置
pub fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// 比较版本号大小
pub fn compareVersion(version1: &str, version2: &str) -> Ordering {
    let nums1: Vec<&str> = version1.split('.').collect();
    let nums2: Vec<&str> = version2.split('.').collect();
    let n1 = nums1.len();
    let n2 = nums2.len();

    // 比较版本
    for i in 0..std::cmp::max(n1, n2) {
        let i1 = if i < n1 {
            nums1[i].parse::<i32>().unwrap()
        } else {
            0
        };
        let i2 = if i < n2 {
            nums2[i].parse::<i32>().unwrap()
        } else {
            0
        };
        if i1 != i2 {
            return if i1 > i2 {
                Ordering::Greater
            } else {
                Ordering::Less
            };
        }
    }
    // 版本相等
    Ordering::Equal
}

// pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
//     (0..s.len())
//         .step_by(2)
//         .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
//         .collect()
// }
//
// pub fn encode_hex(bytes: &[u8]) -> String {
//     let mut s = String::with_capacity(bytes.len() * 2);
//     for &b in bytes {
//         write!(&mut s, "{:02x}", b).unwrap();
//     }
//     s
// }


/// 压缩文件
pub fn compressionFile(filePath: &Path, outputPath: &Path, compressionGrade: u8) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(&filePath)?;
    let mut outputfile = File::create(&outputPath)?;

    // miniz_oxide::deflate::stream::deflate()
    // miniz_oxide::deflate::core::compress()

    // 缓冲区
    let mut buffer = [0u8; BUFFER_SIZE];

    // 循环读取并压缩写出文件
    loop {
        let nbytes = file.read(&mut buffer)?;
        let compressed = miniz_oxide::deflate::compress_to_vec(&*buffer.to_vec(), compressionGrade);
        outputfile.write_all(&compressed[..compressed.len()])?;
        if nbytes < buffer.len() { break; }
    }
    Ok(())
}

/// 还原压缩文件
pub fn decompressFile(filePath: &Path, outputPath: &Path) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(&filePath)?;
    let mut outputfile = File::create(&outputPath)?;

    // 缓冲区
    let mut buffer = [0u8; BUFFER_SIZE];

    // 循环读取并解压写出文件
    loop {
        let nbytes = file.read(&mut buffer)?;
        let decompress = miniz_oxide::inflate::decompress_to_vec(&buffer).unwrap();
        outputfile.write_all(&decompress[..decompress.len()])?;
        if nbytes < buffer.len() { break; }
    }
    Ok(())
}
