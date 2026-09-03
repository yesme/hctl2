# serde_jcs · RFC 8785 JSON 规范化

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-LIB-SERDE-JCS<br>
> 对象：[`serde_jcs` 0.2.0](https://crates.io/crates/serde_jcs/0.2.0)（2026-03-25 发布）；同题对照 [`serde_json_canonicalizer` 0.3.2](https://crates.io/crates/serde_json_canonicalizer/0.3.2)（2026-02-03）、[`json-canon` 0.1.3](https://crates.io/crates/json-canon/0.1.3)（2023-05-13）、[`canonical_json` 0.7.0](https://crates.io/crates/canonical_json/0.7.0)（2026-04-28）<br>
> 许可证：serde_jcs MIT OR Apache-2.0；serde_json_canonicalizer MIT；json-canon Apache-2.0；canonical_json MIT；官方测试向量仓库 cyberphone/json-canonicalization Apache-2.0；配套 ryu-js Apache-2.0 OR BSL-1.0，sha2 MIT OR Apache-2.0

## 定位

[系统边界 §命令与跨服务正确性](../../design/spec/system.md#命令与跨服务正确性)规定：跨模块引用的规范摘要统一是「RFC 8785 JCS 规范化字节的 SHA-256」，摘要字段自身不参与计算，每个领域归属者只定义自己规范对象含哪些字段。库负责把一个 `serde::Serialize` 对象变成 RFC 8785 要求的那一串字节；SHA-256 用 `sha2` 0.11.0。它替代的手写代码是「键排序 + 紧凑输出 + 数字格式化」——这三件事手写几乎一定错：Rust 的 `BTreeMap<String>` 按 UTF-8 字节排序而 RFC 要求按 UTF-16 码元；`serde_json` 默认用 Rust 风格的 ryu 输出浮点（`1e21`、`1e-6`）而 RFC 要求 ECMAScript 风格（`1e+21`、`0.000001`）。

## 上游能力

- **RFC 8785**（Informational，2020-06）：对象键按属性名的 UTF-16 码元升序递归排序；数组保序；不输出任何空白；数字必须可表示为 IEEE 754 双精度并按 ECMAScript `Number::toString` 输出，NaN/Infinity 必须报错；字符串中 U+0000–U+001F 用小写 `\u00xx`（`\b \t \n \f \r` 用短写），`"` 与 `\` 转义，其余包括非 ASCII 一律按字面输出。附录 B 给了 27 个数字样本；附录 D 建议超过 2^53 的整数按 I-JSON（RFC 7493）用字符串表示。
- **serde_jcs 0.2.0**：`to_string / to_vec`；依赖 ryu-js 0.2、serde、serde_json（`float_roundtrip`）；依赖树三平台各 8 个 crate；rust-version 1.85，edition 2024；下载 313 万，近 90 天 220 万。仓库 8 star、21 次提交；2021 年后沉寂，2026-03-25 一天内提交「invalid floats 返回错误」「更新 CI」等并发布 0.2.0（仓库 `Cargo.toml` 仍写 0.1.0，版本号未同步）。测试自带附录 B 27 个数字、附录 C/E 样例和 cyberphone 六组 testdata。issue #1（UTF-16 排序）与 #2（用官方 testdata 测试）已关闭；#3「u64 未按双精度截断」自 2021 年开着——实测 0.2.0 已按双精度截断（见下），该 issue 已过期。
- **serde_json_canonicalizer 0.3.2**：`to_string / to_vec`，定位为 serde_json 同名函数的直接替代；依赖 ryu-js 1.0.x、serde、serde_json（`float_roundtrip`）；依赖树 8 个 crate；下载 647 万，近 90 天 430 万。仓库 21 star，最后提交 2026-02-20；测试含 cyberphone 六组 testdata（input、output、outhex 三份对照）、生成数字集与 RFC 正文示例；cyberphone 官方 README 的实现表把它列为 Rust 实现。开着的 issue：#12 嵌套 NaN/Infinity 被写成 `null` 而不是报错（2026-08-21）、#13 大整数 map 键按双精度取整后碰撞丢条目（2026-08-21）、#4 no_std。README 明说 `arbitrary_precision` 数字会被转成双精度、大数请用字符串。
- **json-canon 0.1.3**：Rust + JS 单仓，ryu-js 0.2.2，Apache-2.0；2023-05-23 后无提交，不再考虑。
- **canonical_json 0.7.0**（Mozilla）：实现的是 gibson042 的 Canonical JSON（Firefox Remote Settings 用），非 ASCII 转 `\uXXXX`、浮点用科学计数——**不是 RFC 8785**，与其他语言的 JCS 实现不互通，排除。

## 候选比较

| 候选 | 版本 / 发布 | 许可证 | 规范 | 浮点格式 | 依赖树 | 官方向量测试 | 维护 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| serde_json_canonicalizer | 0.3.2 / 2026-02-03 | MIT | RFC 8785 | ryu-js 1.0.x | 8 | 六组 testdata + outhex + 附录 B 生成集 | 活跃；cyberphone 列名 |
| serde_jcs | 0.2.0 / 2026-03-25 | MIT OR Apache-2.0 | RFC 8785 | ryu-js 0.2 | 8 | 六组 testdata + 附录 B/C/E | 2026-03 复活，仓库版本号未同步 |
| json-canon | 0.1.3 / 2023-05-13 | Apache-2.0 | RFC 8785 | ryu-js 0.2.2 | — | 引用 cyberphone | 2023-05 停更 |
| canonical_json | 0.7.0 / 2026-04-28 | MIT | gibson042 Canonical JSON | 科学计数 | — | 不适用 | 活跃但规范不同 |
| 手写 | — | — | — | serde_json/ryu，格式不合 RFC | 0 | — | 排序与数字两处必错 |

## 实测（2026-09-03，rustc 1.98.0，本机）

方法：把 cyberphone testdata 的 input 解析成 `serde_json::Value`，分别经两个 crate 序列化，与 output 逐字节比对；附录 B 用 IEEE 754 位模式构造 `f64`；另加 HCTL2 关心的边界用例。两个 crate 在全部用例上输出**完全一致**。

| 用例 | 期望 | serde_jcs 0.2.0 | serde_json_canonicalizer 0.3.2 |
| --- | --- | --- | --- |
| arrays / french / structures / unicode / values / weird 六组 | 与 output 逐字节相同 | 全过 | 全过 |
| 附录 B 25 个有限数（含 −0 → `0`、`5e-324`、`1e+21`、`0.000001`、`9.999999999999997e-7`、`1424953923781206.2`） | 逐条相同 | 全过 | 全过 |
| 附录 B NaN、Infinity（顶层） | 报错 | `Err(invalid float value)` | `Err(NaN and +/-Infinity are not permitted)` |
| `u64` 2^53+1、`u64::MAX`、`i64::MIN` | 按双精度：`9007199254740992`、`18446744073709552000`、`-9223372036854776000` | 相同 | 相同 |
| 键 U+FB33、U+1F602、U+E000 | UTF-16 序：U+1F602 < U+E000 < U+FB33（UTF-8 序会把 U+1F602 排最后） | 正确 | 正确 |
| U+007F、U+2028、U+2029、`/` 按字面；U+0000、U+001F、U+0001 转义 | 前四个不转义；后三个为 `\u0000` `\u001f` `\u0001` | 正确 | 正确 |
| `Vec<f64>` 含 NaN；map 值为 Infinity | 应报错 | **写成 `null`** | **写成 `null`**（即 issue #12） |
| `BTreeMap<u64, _>` 两个键 2^53 与 2^53+1 | 应报错或保留两条 | **静默合并，保留前一条** | **静默合并，保留后一条**（即 issue #13） |

## 边界与取舍

数字的坑：

1. 整数超过 2^53 会被无声地按双精度取整（两个 crate 都如此，也是 RFC 的要求）。HCTL2 规范对象里的代次、序号如果可能超过 2^53（`u64` 计数器理论上可以），按 RFC 7493 用字符串承载；更简单的做法是约束层规定规范对象里的整数字段不超过 2^53，并在契约测试里放一条超界拒绝用例。
2. 顶层 NaN/Infinity 报错，但嵌在数组或对象里的 NaN/Infinity 两个 crate 都写成 `null`——摘要算得出来但对象已经变了。规范对象里干脆不放 `f64`；如果必须放，序列化前自己校验有限性。
3. `serde_json` 的 `arbitrary_precision` feature 一旦被工作区任何 crate 打开就全局生效，规范化时任意精度数字会被转成双精度。工作区里禁用该 feature，或在 CT 里用一个大数字面值守住。
4. 非字符串 map 键：serde 会把它们转成字符串键，取整后可能碰撞并丢条目。规范对象的键只用字符串标识符。
5. 浮点格式由 ryu-js 负责（ECMAScript 语义），不要用 `serde_json` 直接输出的浮点字节去比对。

Unicode 的坑：

1. 排序按 UTF-16 码元，不是码点也不是 UTF-8 字节：U+E000–U+FFFF 排在所有增补平面字符（代理对以 0xD800–0xDBFF 开头）之后。两个 crate 都处理对了，手写 `BTreeMap<String>` 必错。
2. 非 ASCII 一律字面输出、不转义（与 Mozilla canonical_json 相反）；U+007F 和 U+2028/U+2029 也按字面。
3. JCS 不做 Unicode 规范化（testdata `unicode.json` 的未规范化 `Å` 原样保留）。同一个名字的 NFC 与 NFD 形式摘要不同——是否在进入规范对象前做 NFC，由领域归属者定义字段时决定。
4. 可选字段「缺席」与「为 `null`」字节不同：规范对象定义要写清哪些字段总是出现。

契约测试怎么钉住 RFC 8785 官方向量：

- **向量在哪里**：仓库 [cyberphone/json-canonicalization](https://github.com/cyberphone/json-canonicalization)（Apache-2.0）的 `testdata/input/*.json`（6 个文件：arrays、french、structures、unicode、values、weird）、`testdata/output/*.json`（期望字节）、`testdata/outhex/*.txt`（同一期望的十六进制）；最后提交 `19d51d7`（2024-12-13）。RFC 正文附录 B 的 27 行数字表、§3.2.3 的排序示例也是向量。ES6 数字大集 `es6testfile100m.txt.gz`（1 亿行、约 4.0 GB）作为 release 资产发布，且提供确定性生成器 `numgen.go / numgen.js` 与前 10^3–10^8 行的 SHA-256 表（前 1000 行：`be18b62b…8687`，37,967 字节）。
- **钉法**：把六组 input/output/outhex 和附录 B 表按上游提交号复制进 CT fixture（附上 Apache-2.0 声明）；CT 断言 `to_vec` 与 output 逐字节相等；数字用位模式构造。再加 HCTL2 自己的负向用例：嵌套 NaN 拒绝、整数超 2^53 拒绝、非字符串键拒绝、`arbitrary_precision` 未开启。ES6 数字集取本地生成的前 1000 行即可，不下载 4 GB。CT 只钉向量与 HCTL2 规范对象，不钉具体 crate——两者可互换。

## 决定建议

**采用 SDK**。两个 crate 在全部官方向量与边界用例上字节一致，可互换。首选 **serde_json_canonicalizer 0.3.2**：cyberphone 官方列名的 Rust 实现、测试套件自带三份对照的 testdata、ryu-js 用现行 1.x 线、下载量两倍于对手。**serde_jcs 0.2.0** 为备选，实测同样全过；评审裁决包写的「用 serde_jcs」不违反任何向量，是否换成前者属实现层选择，不改约束。两者共有的两个缺陷（嵌套 NaN 写 `null`、大整数键碰撞）由 HCTL2 规范对象的定义与契约测试兜住，不等上游修。不采用 canonical_json（规范不同）和 json-canon（停更），禁止手写规范化。摘要用 `sha2` 0.11.0。

## 证据

- 规范：[RFC 8785](https://www.rfc-editor.org/rfc/rfc8785)（§3.2.2.3 数字、§3.2.2.2 字符串、§3.2.3 排序、附录 B 数字样本、附录 D I-JSON）、[RFC 7493 I-JSON](https://www.rfc-editor.org/rfc/rfc7493)
- 官方向量：[cyberphone/json-canonicalization `testdata`](https://github.com/cyberphone/json-canonicalization/tree/19d51d7fe467d4706a3ff08adf8a748f29fc21e0/testdata)（README 含 ES6 数字集生成算法与 SHA-256 表）、[es6testfile100m.txt.gz](https://github.com/cyberphone/json-canonicalization/releases/download/es6testfile/es6testfile100m.txt.gz)、[仓库 LICENSE（Apache-2.0）](https://github.com/cyberphone/json-canonicalization/blob/master/LICENSE)
- serde_jcs：[crates.io 0.2.0](https://crates.io/crates/serde_jcs/0.2.0)、[仓库](https://github.com/l1h3r/serde_jcs)、[tests/basic.rs](https://github.com/l1h3r/serde_jcs/blob/main/tests/basic.rs)、[issue #1 UTF-16 排序](https://github.com/l1h3r/serde_jcs/issues/1)、[issue #3 数字精度](https://github.com/l1h3r/serde_jcs/issues/3)
- serde_json_canonicalizer：[crates.io 0.3.2](https://crates.io/crates/serde_json_canonicalizer/0.3.2)、[仓库与 README](https://github.com/evik42/serde-json-canonicalizer)、[tests/testdata.rs](https://github.com/evik42/serde-json-canonicalizer/blob/main/tests/testdata.rs)、[issue #12 嵌套 NaN](https://github.com/evik42/serde-json-canonicalizer/issues/12)、[issue #13 大整数键碰撞](https://github.com/evik42/serde-json-canonicalizer/issues/13)
- 其他候选：[json-canon](https://github.com/ahdinosaur/json-canon)、[canonicaljson-rs（gibson042 规范）](https://github.com/mozilla-services/canonicaljson-rs)、[ryu-js](https://crates.io/crates/ryu-js)、[sha2 0.11.0](https://crates.io/crates/sha2/0.11.0)
- 本库：[系统边界 §命令与跨服务正确性](../../design/spec/system.md#命令与跨服务正确性)、[契约测试 CT-SYSTEM JCS 条目](../../design/contract-tests.md)、[部件矩阵表 D 内容摘要一行](../component-matrix-20260902.md#d--约束层里靠第一方代码实现的通用机制)
