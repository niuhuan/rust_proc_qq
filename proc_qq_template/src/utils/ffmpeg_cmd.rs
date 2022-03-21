use std::process::{Command, Stdio};

pub(crate) fn ffmpeg_run_version() -> anyhow::Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.stderr(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.arg("-version");
    match cmd.status() {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::Error::msg("未找到ffmpeg, 请先安装ffmpeg.")),
    }
}

/// 合并音频视频
pub(crate) fn ffmpeg_convert(input: &str, output: &str) -> anyhow::Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.stderr(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.arg("-i");
    cmd.arg(input);
    cmd.arg("-f");
    cmd.arg("s16le");
    cmd.arg("-ar");
    cmd.arg("24000");
    cmd.arg("-ac");
    cmd.arg("1");
    cmd.arg(output);
    let status = cmd.status().unwrap();
    if status.code().unwrap() == 0 {
        Ok(())
    } else {
        anyhow::Result::Err(anyhow::Error::msg(format!(
            "FFMPEG 未能成功运行 : EXIT CODE : {}",
            status.code().unwrap()
        )))
    }
}
