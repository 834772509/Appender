# Appender

[简体中文](README.zh.md) [English](README.md)

## Introduction

`Appender` is a tool for adding, reading and exporting additional resources.

### What is the use of `Appender`?

- The most typical is that some software can generate exe files from some data stream files, such as some mp3 generators, Flash generators, and S-demo for animation. Their function is to bind data to PE;
- Can be used for installation package production, use this program to attach the installation package to a custom program, and release resources during installation;
- Can be used to hide files, such as adding files to pictures and other formats;

### Will the increased resources occupy the running memory?

-`Overlay` is appended to the back of the file and is not mapped to the data in the memory space. It provides its own program to open it for reading -`Overlay` is just data, it is not mapped to memory, it will be read by the program in its own way

### How much resources can be increased?

- 4GB is a hard limit for all portable executable programs (32-bit and 64-bit PE)
- Other formats (such as picture formats) generally do not have this restriction

### How to ensure the integrity of resources?

`Appender` will check if the resource length is consistent before releasing the file, and will also perform a second check after release.

## Use

We use the `resource ID` to mark the file. The `resource ID` can be any text less than 64 in length, and no repetition is allowed.

### Increase resources

`Appender.exe add targetFile resourceFile resourceID [newFile]`

- Basic usage: `Appender.exe add D:\Program.exe D:\file.zip Archive`
- Output new file: `Appender.exe add D:\Program.exe D:\file.zip Archive D:\Program-new.exe`

### Release resources

`Appender.exe export targetFile resourceID outputPath`

- Specify the output path (keep the original file name): `Appender.exe export D:\Program.exe Archive D:\`
- Specify the output path (custom file name) `Appender.exe export D:\Program.exe Archive D:\file.zip`
- Export to the target file directory: `Appender.exe export D:\Program.exe Archive file.zip`
