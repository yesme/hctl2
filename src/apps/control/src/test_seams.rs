//! 第 9 包验收第 4 条的测试缝：杀点要落在精确的状态上，sleep 只能猜。
//!
//! 生产不设 `HCTL2_TEST_SEAM_ROOT`，`held` 恒为 false；测试把它指到一个目录，目录里出现
//! 同名标记文件时，对应的尝试停在验收要的那个状态（与平台失联时同形状的 Attention），
//! 等测试杀进程、删标记、再起。标记是文件，不是计时器：删掉之前状态不会前进，删掉之后
//! 下一次尝试按既有恢复路径继续。缝只改「这一次尝试停在哪里」，不改任何判定。

/// `true` while the named marker exists under the seam root.
pub(crate) fn held(name: &str) -> bool {
    std::env::var_os("HCTL2_TEST_SEAM_ROOT")
        .is_some_and(|root| std::path::Path::new(&root).join(name).exists())
}
