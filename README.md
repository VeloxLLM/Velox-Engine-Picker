# Velox Engine Picker

Velox Engine Picker 是一个 Tauri + React + Rust 桌面工具，用于检测本机硬件与常见推理运行时，并分别给出：

- **理论最佳方案**：硬件兼容情况下的推荐引擎与后端。
- **当前可运行首选**：已经检测到所需驱动和运行时的方案。

项目地址：[github.com/VeloxLLM/Velox-Engine-Picker](https://github.com/VeloxLLM/Velox-Engine-Picker)

## 平台支持

| Feature | 平台 | 状态 |
| --- | --- | --- |
| `v1`（默认） | Windows x64 | 稳定首发目标 |
| `v2` | Windows、macOS x64/ARM64 | 实验性 |
| `v3` | Windows、macOS、Linux，含 ARM | 实验性 |

非法的平台/feature 组合会在编译时给出错误。v2/v3 通过 CI 做编译检查，但不代表已经完成全部真实硬件验证。

## 已实现能力

- Windows 使用 DXGI 枚举真实图形适配器、过滤软件适配器并读取专用显存；无法读取时明确显示“未知”。
- 展示 CPU 物理核心、逻辑处理器、架构和 AVX/AVX2/AVX-512/NEON 等指令集。
- 区分 GPU 未发现和 GPU 检测失败。
- 探测 OpenVINO、CUDA/NVIDIA 驱动、TensorRT、DirectML、ROCm/HIP 和 llama.cpp 的本地信号。
- 运行时状态分为 `Ready`、`CompatibleMissingRuntime`、`Unsupported`、`Unknown`。
- 推荐规则按操作系统过滤；Windows AMD 不会仅凭厂商直接首推 ROCm。

运行时探测是保守的本地检查，不会执行或自动安装第三方程序。检测到文件也不等价于已经验证所有模型都能运行。

## 开发

要求：Node.js 20 LTS、Rust stable、Windows 上的 Visual Studio C++ Build Tools。

```powershell
npm ci
npm test
npm run build

# Windows x64 稳定版
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features --features v1
npm run tauri dev -- --features v1
```

实验版本：

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --no-default-features --features v2
cargo check --manifest-path src-tauri/Cargo.toml --no-default-features --features v3
```

## 发布验收

- 自动化：前端组件测试、Rust 单元测试、Clippy、各 feature 编译检查、Windows Tauri 构建。
- 人工：至少验证 Intel 核显和 NVIDIA 独显；AMD 在完成实机验证前保持“未完整验证”标记。
- 无签名证书时仅发布明确标注的 unsigned prerelease。

## 当前限制与路线图

- v2/v3 仍是实验性支持。
- 尚未按模型规模、量化格式和上下文长度精确估算内存。
- Markdown/JSON 报告导出与可复现性能基准留待后续版本。
- 软件运行时版本识别目前以路径和环境信号为主，后续会增加更精确的版本查询。

## License

[MIT](LICENSE)
