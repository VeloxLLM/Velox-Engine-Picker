# Velox Engine Picker

Velox Engine Picker 是一个 Tauri + React + Rust 桌面工具，用于检测本机硬件、平台和常见推理运行时，并分别给出：

- **当前可运行首选**：已经检测到所需驱动或运行时的方案。
- **理论最佳方案**：硬件兼容，但可能仍需安装运行时的方案。

项目不会下载、安装或执行第三方运行时，也不会上传硬件信息。

## 平台分层

| Feature | 平台 | 发布状态 |
| --- | --- | --- |
| `v1`（默认） | Windows x64 | 稳定首发目标 |
| `v2` | Windows、macOS x64/ARM64 | 实验性 |
| `v3` | Windows、macOS、Linux，含 ARM | 实验性 |

非法平台/feature 组合会在编译期失败。v2/v3 在原生 CI runner 做编译和单元测试，但在完成对应真实硬件验收前不声明稳定支持。

## 已实现

### 硬件与平台检测

- Windows 通过 DXGI 枚举适配器，过滤软件、远程、虚拟和间接显示适配器，并读取真实专用显存。
- 界面分别列出 dGPU 与 iGPU；未发现 iGPU 时会明确提示，而不是静默省略。
- 无法取得显存时显示“未知”，不会根据显卡名称猜容量或套用低显存结论。
- 显示 CPU 物理核心、逻辑处理器、架构和 SSE4.2/AVX/AVX2/AVX-512/FMA/NEON 等指令集。
- GPU 未发现返回正常 CPU-only 结果；GPU 探测失败返回结构化错误，前端允许重试。
- 检测在后台线程执行，不阻塞 UI 主线程。

### 运行时可用性

每个后端使用四级状态：

| 状态 | 含义 |
| --- | --- |
| `Ready` | 硬件和所需驱动/运行时均检测到 |
| `CompatibleMissingRuntime` | 硬件兼容，但软件尚未检测到 |
| `Unsupported` | 当前平台或硬件不支持 |
| `Unknown` | 检测失败，不能武断推荐 |

Windows v1 探测 OpenVINO、NVIDIA 驱动/CUDA、TensorRT、DirectML/D3D12、ROCm/HIP、llama.cpp 和 ONNX Runtime 的保守本地信号。检测到文件或命令只代表“具备运行条件信号”，不等价于所有模型均已验证。

运行时信号仍用于生成“当前可运行首选”，但界面不再展示冗长的逐项运行时列表。

### 在线模型推荐

- 左侧“在线模型”独立页面实时读取 Ollama Library 与 LM Studio Model Catalog 官方目录，不扫描本机模型文件。
- 从在线目录识别模型族和参数规格，按常见 Q4 量化体积、独显显存和系统内存筛选本机可能运行的规格。
- 可按 Ollama / LM Studio 来源和模型名称筛选；单个来源读取失败时保留另一来源结果并明确提示。
- 估算不代表推理速度、最大上下文或模型一定兼容；硬件信息只在本机参与计算，不会随目录请求上传。

### 平台感知推荐

- Windows AMD 不会仅凭厂商首推 ROCm；DirectML 是默认理论方案，ROCm 只有检测到有效环境时才进入可运行候选。
- DirectML 不会出现在非 Windows 平台。
- NVIDIA 未知显存不会被当作低显存。
- Intel 核显、NVIDIA 已知/未知显存、AMD Windows、CPU-only、运行时缺失、探测失败和 v1/v2/v3 规则均由固定夹具或原生 CI 覆盖。

## 界面状态

界面覆盖首次加载、检测失败、重新检测、无 GPU、未检测到 iGPU、未知显存、当前无可运行首选、在线模型估算以及理论方案展示；支持键盘操作、窄窗口布局和高对比度状态标签。

Shell 插件和无用权限已移除，应用采用最小 capability 与 CSP；当前版本不需要打开外部链接。

## 开发与验证

固定环境：Node.js 20 LTS、Rust stable `1.97.1`。Windows 原生构建需要 Visual Studio C++ Build Tools。

```powershell
npm ci
npm test
npm run build

cargo fmt --manifest-path src-tauri\Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri\Cargo.toml `
  --no-default-features --features v1 --all-targets -- -D warnings
cargo test --locked --manifest-path src-tauri\Cargo.toml `
  --no-default-features --features v1

npm run tauri build -- --features v1
```

实验版检查：

```powershell
cargo test --locked --manifest-path src-tauri\Cargo.toml --no-default-features --features v2
cargo test --locked --manifest-path src-tauri\Cargo.toml --no-default-features --features v3
```

`package-lock.json` 和 `src-tauri/Cargo.lock` 均已提交。CI 将 Windows v1 前端、Rust 测试和 Tauri 构建作为门禁；macOS v2 与 Linux v3 使用对应原生 runner 做实验性检查。

## 发布

Tauri 配置显式启用 Windows MSI/NSIS，Windows MSVC 构建采用静态 CRT。推送 `v*` 标签会构建两种安装包、生成 `SHA256SUMS.txt`，并创建 unsigned prerelease。发布前仍需人工覆盖：

1. Intel 核显机器。
2. NVIDIA 独显机器。
3. AMD 机器；无法取得时必须标注“未实机验证”。
4. 干净 Windows x64 环境的安装、启动、重新检测和卸载。

没有代码签名证书时不得把 unsigned prerelease 描述为正式签名稳定版。

### 2026-08-01 本机验证记录

- 前端组件/Hook 测试与生产构建通过；生产依赖 `npm audit --omit=dev` 为 0 个已知漏洞。
- Windows v1、v2、v3 固定夹具测试和 v1 严格 Clippy 通过。
- Windows v1 静态 CRT Release 构建通过；PE 依赖检查不包含 `VCRUNTIME` 或 `MSVCP` DLL。
- WiX MSI 与 NSIS setup 均已实际生成并计算 SHA-256。

上述记录不能替代 Intel/NVIDIA/AMD 三类真实机器和干净 Windows 安装/卸载验收，因此当前标签发布仍保持 unsigned prerelease。

## 已知限制与路线图

- v2/v3 仍为实验性。
- 当前不运行自动性能基准，避免把一次短测当作通用引擎排名。
- 内存建议仍是保守区间；后续将允许输入模型规模、量化格式和上下文长度。
- 后续计划导出 Markdown/JSON 硬件报告，并增加官方运行时安装链接；首版不会自动安装第三方组件。

## License

[MIT](LICENSE)
