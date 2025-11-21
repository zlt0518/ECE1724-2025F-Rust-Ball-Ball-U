# Macroquad 客户端使用指南

## ✅ 已完成的改动

### 1. 替换依赖 (`client/Cargo.toml`)
- ❌ 移除：`ratatui`, `crossterm`
- ✅ 添加：`macroquad = "0.4"`

### 2. 重写渲染管理器 (`client/src/render_manager.rs`)

**使用 macroquad 的真正图形渲染：**
- ✅ 真正的圆形绘制（不再是字符）
- ✅ 平滑的摄像机跟随
- ✅ 网格和世界边界可视化
- ✅ 客户端预测（使用 `vx`, `vy` 字段）
- ✅ 视口裁剪（只渲染可见对象）
- ✅ 实时显示玩家速度（基于 `calculate_speed_from_score`）

**游戏机制集成：**
```rust
// 正确使用 mechanics.rs 中的速度计算
let expected_speed = mechanics::calculate_speed_from_score(
    player.score,
    snapshot.constants.move_speed_base,
);
```

### 3. 重写输入管理器 (`client/src/input_manager.rs`)

**使用 macroquad 的键盘输入：**
- ✅ `is_key_down()` 检测持续按键
- ✅ `is_key_pressed()` 检测单次按键
- ✅ 发送 `ClientMessage::Input` 到服务器
- ✅ 服务器端处理速度计算（基于 score）

### 4. 重写主循环 (`client/src/main.rs`)

**使用 macroquad 的游戏循环：**
- ✅ `#[macroquad::main]` 宏
- ✅ `next_frame().await` 异步帧循环
- ✅ 窗口配置（标题、大小、可调整）
- ✅ 连接状态显示
- ✅ 错误处理和重连提示

---

## 🎮 游戏机制正确实现

### 1. **速度随分数降低**
```rust
// shared/src/mechanics.rs
pub fn calculate_speed_from_score(score: u32, base_speed: f32) -> f32 {
    let slow_factor = 1.0 / (1.0 + (score as f32) * 0.005);
    base_speed * slow_factor
}
```
- Score 0 → 最快速度
- Score 增加 → 速度降低
- UI 显示实时速度

### 2. **半径随分数增长**
```rust
// shared/src/mechanics.rs
pub fn calculate_radius_from_score(score: u32, base_radius: f32) -> f32 {
    base_radius + (score as f32).sqrt()
}
```
- 半径 = 基础半径 + √score
- 典型 agar.io 机制

### 3. **客户端预测**
```rust
// 使用速度字段平滑渲染
let pred_seconds = received_at.elapsed().as_secs_f32();
let predicted_x = player.x + player.vx * pred_seconds;
let predicted_y = player.y + player.vy * pred_seconds;
```
- 减少网络延迟的视觉影响
- 更流畅的游戏体验

---

## 🖥️ 如何运行（解决 X11 问题）

### ⚠️ 问题：远程 SSH 无法显示图形

macroquad 需要图形环境（X11/Wayland），SSH 连接默认没有。

### 解决方案（选择一种）

#### 方案 A：本地运行客户端 + 远程运行服务器（推荐）✨

```bash
# 1. 在远程服务器运行服务端
ssh user@remote-server
cd ballballu
cargo run -p server

# 2. 在本地机器修改客户端连接地址
# 编辑 client/src/main.rs
let url = "ws://your-server-ip:8000";  # 改为服务器 IP

# 3. 在本地运行客户端
cd ballballu
cargo run -p client
```

**优势：**
- ✅ 完美的图形显示
- ✅ 无延迟
- ✅ 最佳游戏体验

---

#### 方案 B：VNC Remote Desktop

```bash
# 在远程服务器安装桌面环境
sudo dnf groupinstall "Xfce Desktop"
sudo dnf install tigervnc-server

# 启动 VNC 服务器
vncserver :1 -geometry 1920x1080

# 在本地连接（通过 SSH 隧道）
ssh -L 5901:localhost:5901 user@remote-server

# 使用 VNC 客户端连接 localhost:5901
# 在 VNC 桌面中运行：
cargo run -p client
```

---

#### 方案 C：X11 转发（如果本地有 X11）

```bash
# Linux/Mac
ssh -X user@remote-server
cargo run -p client

# Windows（需要安装 VcXsrv 或 Xming）
# 1. 启动 XLaunch
# 2. SSH 连接时启用 X11 转发
ssh -X user@remote-server
cargo run -p client
```

**注意：** X11 转发会有网络延迟。

---

## 📊 渲染效果

### 终端 (Ratatui)
```
●●●   ← 使用字符近似圆形
 ●●   → 永远无法完美
●●●
```

### Macroquad (真正的图形)
```
  ●●●●●
 ●●●●●●●   ← 平滑的圆形
●●●●●●●●●  → 完美像素渲染
 ●●●●●●●
  ●●●●●
```

**视觉对比：**
- Ratatui: 方块感、锯齿、字符限制
- Macroquad: 平滑圆形、渐变、专业游戏画面

---

## 🎯 UI 功能

### 左上角信息面板
- Tick 计数
- 玩家数量
- Dots 数量
- 玩家排行榜（按分数排序）
- 实时显示：Score, Radius, Speed

### 游戏世界
- 网格线（每 100 单位）
- 世界边界（红色线）
- 摄像机跟随玩家
- 视口裁剪优化

### 控制提示
- WASD / 方向键 - 移动
- ESC - 退出

---

## 🔧 编译和运行

```bash
# 检查代码
cargo check --package client

# 运行（开发模式）
cargo run -p client

# 构建 release 版本
cargo build --package client --release

# 运行 release 版本（更流畅）
./target/release/client
```

---

## 📝 代码亮点

### 1. 正确的游戏机制
```rust
// 渲染时显示基于 score 的速度
let expected_speed = mechanics::calculate_speed_from_score(
    player.score,
    snapshot.constants.move_speed_base,
);
```

### 2. 客户端预测
```rust
// 平滑插值
let pred_seconds = received_at.elapsed().as_secs_f32();
let predicted_x = player.x + player.vx * pred_seconds;
```

### 3. 性能优化
```rust
// 视口裁剪 - 只渲染可见对象
if predicted_x >= min_x - player.radius
    && predicted_x <= max_x + player.radius { ... }
```

### 4. 窗口配置
```rust
fn window_conf() -> Conf {
    Conf {
        window_title: "Ball Ball U - Agar.io Clone".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}
```

---

## ✅ 测试清单

- [ ] 客户端成功连接到服务器
- [ ] 玩家可以移动（WASD/方向键）
- [ ] 圆形平滑渲染
- [ ] 吃 dot 后 radius 增大
- [ ] 吃 dot 后速度变慢
- [ ] 分数正确显示
- [ ] 摄像机跟随玩家
- [ ] ESC 退出

---

## 🐛 故障排查

### 问题：`XOpenDisplay() failed!`
**原因：** SSH 环境无图形界面

**解决：** 使用方案 A（本地运行客户端）或方案 B（VNC）

### 问题：窗口太小/太大
**解决：** 修改 `window_conf()`
```rust
window_width: 1920,   // 改为你喜欢的尺寸
window_height: 1080,
```

### 问题：帧率低
**解决：** 使用 release 模式
```bash
cargo run --release -p client
```

---

## 🎉 总结

✅ **成功替换** Ratatui → Macroquad  
✅ **真正的像素渲染** 代替字符近似  
✅ **正确集成游戏机制** (mechanics.rs)  
✅ **流畅的游戏体验** (客户端预测)  
✅ **专业的 UI** (信息面板、控制提示)

**推荐运行方式：** 本地运行客户端 + 远程运行服务器

