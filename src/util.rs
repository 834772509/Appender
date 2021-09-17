use std::path::Path;
use std::fs::File;
use std::io::{BufReader};
use std::error::Error;
use flate2::write::GzEncoder;
use flate2::write::GzDecoder;
use flate2::Compression;
use std::io::copy;

/// 压缩文件
/// #参数
/// 1. 文件路径
/// 2. 输出路径
/// 3. 压缩等级(0-9)
///     - 0: 不压缩
///     - 1: 为优化编码的最佳速度
///     - 9: 针对正在编码的数据大小进行优化。
pub fn compressionFile(filePath: &Path, outputPath: &Path, compressionGrade: u32) -> Result<(), Box<dyn Error>> {
    let mut input = BufReader::new(File::open(filePath)?);
    let output = File::create(outputPath)?;
    let mut encoder = GzEncoder::new(output, Compression::new(compressionGrade));
    copy(&mut input, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

/// 还原压缩文件
/// #参数
/// 1. 文件路径
/// 2. 输出路径
pub fn decompressFile(filePath: &Path, outputPath: &Path) -> Result<(), Box<dyn Error>> {
    let mut input = BufReader::new(File::open(filePath)?);
    let output = File::create(outputPath)?;
    let mut decoder = GzDecoder::new(output);
    copy(&mut input, &mut decoder)?;
    decoder.finish()?;
    Ok(())
}
