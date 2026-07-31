# ⚡ Velox Engine Picker

> **方案 B — 项目名称：Velox-Engine-Picker**
>
> 仓库：[github.com/VeloxLLM/Velox-Engine-Picker](https://github.com/VeloxLLM/Velox-Engine-Picker)

---

## 项目简介

### 🇨🇳 中文
**Velox Engine Picker** — 一键检测本机 CPU / iGPU / dGPU 硬件配置，智能为你的 LLM 模型推荐最佳推理引擎（OpenVINO · CUDA · TensorRT · ROCm · llama.cpp）。基于 Rust + egui 打造的跨平台桌面工具。

### 🇺🇸 English
**Velox Engine Picker** — A cross-platform Rust GUI tool that detects your local CPU, iGPU, and discrete GPU configurations with one click, and intelligently recommends the optimal LLM inference engine (OpenVINO, CUDA, TensorRT, ROCm, llama.cpp, and more) for your hardware.

### 🌐 中英双语
**Velox Engine Picker** — 一键检测本机 CPU / 核显 / 独显硬件配置，智能为你的 LLM 模型推荐最佳推理引擎（OpenVINO · CUDA · TensorRT · ROCm · llama.cpp）。  
A cross-platform Rust GUI utility that detects CPU/iGPU/dGPU specs in one click and recommends the best LLM inference backend for your machine.

---

## ✨ 特性

- 🔍 **一键硬件检测**：CPU 型号 / 核心数 / 频率 / 架构、内存使用率进度条、GPU/iGPU 列表（厂商 / 类型 / 后端 / 显存估算）
- 🎯 **智能引擎推荐**：
  - NVIDIA dGPU（显存充足）→ TensorRT
  - NVIDIA dGPU（显存较小）→ CUDA + llama.cpp
  - AMD dGPU → ROCm / DirectML
  - Intel GPU（iGPU / Arc）→ OpenVINO GPU
  - Apple Silicon → llama.cpp + Metal
  - ARM CPU（v3）→ llama.cpp (ARM64 + NEON)
  - 兜底 → CPU (OpenVINO / llama.cpp)
- 📋 **版本分层**：通过 Cargo features 提供 v1 / v2 / v3 三个递进版本（见下表）
- 💡 **模型大小建议**：根据显存/内存自动给出可运行的模型规模
- 🎨 **纯 Rust 即时 GUI (eframe/egui)**：启动快、跨平台、单一 exe 分发

---

## 🗂️ 版本分层（v1 / v2 / v3）

使用 **Cargo features** 实现递进式包含（v3 ⊇ v2 ⊇ v1）：

| 版本 | Feature | 目标平台 | 图形后端 | 额外能力 |
|------|---------|---------|----------|---------|
| **v1** (默认) | `v1` | 🪟 **Windows 专用** | Dx12 / Dx11 / Vulkan | DirectML 优先推荐，过滤 Metal |
| **v2** | `v2` | 🪟 Windows + 🍎 **macOS** (AMD64) | 上述 + **Metal** | Apple Silicon Metal 推荐分支；macOS UI 适配 |
| **v3** | `v3` | 上述 + ⚛️ **全架构** (AArch64 / ARM64 / ARM32) | PRIMARY（全后端） | **ARM 架构感知**：NEON 优化提示、Snapdragon X / 树莓派 / Apple Silicon 自动识别 |

> 💡 v3 是"完整版"，v2 在 v1 上加 macOS，v1 专注最广泛的 Windows 用户。

---

## 🚀 快速开始

### 前置要求
- **Rust toolchain**：Rust 1.75+（通过 [`rustup`](https://rustup.rs/) 安装）
- **Windows**：需安装最新的显卡驱动（Vulkan 1.2+ / DirectX 12）
- **macOS**（v2/v3）：Xcode Command Line Tools
- **Linux ARM**（v3）：`libxcb` / `libxkbcommon` / Vulkan driver

### 快捷命令（推荐）

项目已在 `.cargo/config.toml` 预置快捷命令：

```bash
# --- 运行调试版 ---
cargo v1     # 默认：Win-only
cargo v2     # Win + macOS
cargo v3     # 完整版：Win + macOS + ARM

# --- 构建 Release 版 ---
cargo build-v1
cargo build-v2
cargo build-v3

# --- 快速检查（不链接）---
cargo check-v3
```

### 等价手动命令

```bash
# Win-only (v1)
cargo run --features v1
cargo build --release --features v1

# Win + macOS (v2)
cargo run --features v2
cargo build --release --features v2

# Full (v3，包含 ARM 感知)
cargo run --features v3
cargo build --release --features v3

# ARM64 用户可附加原生 CPU 优化（可选，可获得 ~15% CPU 推理加速）
RUSTFLAGS="-C target-cpu=native" cargo build-v3
```

### 交叉编译（常用 target）

需要先 `rustup target add <target>`：

```bash
# macOS (Apple Silicon)
cargo build-v3 --target aarch64-apple-darwin

# macOS (Intel)
cargo build-v2 --target x86_64-apple-darwin

# Linux ARM64 (树莓派5 / ARM服务器)
cargo build-v3 --target aarch64-unknown-linux-gnu

# Windows ARM64 (Surface / Snapdragon X Elite)
cargo build-v3 --target aarch64-pc-windows-msvc
```

---

## 🧱 项目结构

```
Velox-Engine-Picker/
├── Cargo.toml                  # features: v1 / v2 / v3
├── .cargo/config.toml          # cargo aliases (cargo v1/v2/v3)
├── .gitignore
├── README.md                   ← 本文件
└── src/
    ├── main.rs                 # 入口：启动 eframe
    ├── hardware/               # 硬件检测模块
    │   ├── mod.rs              # HardwareInfo 汇总结构
    │   ├── cpu.rs              # CPU（sysinfo + 架构枚举 CpuArch）
    │   ├── memory.rs           # 内存采集
    │   └── gpu.rs              # GPU（wgpu，wgpu 后端按 feature 选择）
    ├── engine/                 # 推荐算法模块
    │   ├── mod.rs
    │   ├── types.rs            # 引擎/后端枚举（6 引擎 · 8 后端）
    │   └── recommender.rs      # 推荐决策（按 feature 过滤 DirectML/Metal/ARM 提示）
    └── ui/                     # egui 界面
        ├── mod.rs
        ├── app.rs              # 主窗口 + 版本徽章 + 重新检测
        ├── hardware_panel.rs   # CPU 卡片 / 内存进度条 / GPU 列表
        └── recommendation_panel.rs  # 首推大卡片 + 理由 + 备选 + 速查表
```

---

## 🧪 引擎推荐决策表（内置逻辑）

| 硬件场景 | 首推方案 | 备选方案 |
|---------|---------|---------|
| NVIDIA dGPU 显存 ≥ 4GB | **TensorRT** | CUDA (llama.cpp) · DirectML |
| NVIDIA dGPU 显存 < 4GB | **CUDA (llama.cpp)** | TensorRT · DirectML |
| AMD dGPU | **ROCm** | DirectML · Vulkan (llama.cpp) |
| Intel iGPU / Arc | **OpenVINO GPU** | DirectML · OpenVINO CPU |
| Apple Silicon (v2+) | **llama.cpp + Metal** | ONNX Runtime CPU |
| ARM CPU 无 GPU (v3) | **llama.cpp (ARM64 + NEON)** | OpenVINO CPU |
| 纯 CPU x86_64 | **OpenVINO CPU** | llama.cpp CPU |

---

## 📝 开发计划

- [x] v1：Windows 专用版本
- [x] v2：加入 macOS + Metal 支持
- [x] v3：加入 ARM 架构感知
- [ ] 更准确的 VRAM 检测（DXGI / NVML / IOKit 平台原生 API）
- [ ] CPU 指令集检测（AVX2 / AVX-512 / NEON / AMX）
- [ ] 导出推荐报告（Markdown / JSON）
- [ ] 提供预编译 GitHub Actions CI 发布包

---

## 📄 License

MIT OR Apache-2.0 © Velox
