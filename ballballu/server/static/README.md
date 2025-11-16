# Ball Ball U Server Test Client

这是一个用于测试 Ball Ball U WebSocket 服务器的前端测试页面。

## 📁 文件结构

```
static/
├── test.html      # HTML 结构文件
├── styles.css     # CSS 样式文件
├── app.js         # JavaScript 逻辑文件
└── README.md      # 本说明文件
```

代码已按关注点分离：
- **test.html**: 只包含 HTML 结构和页面布局
- **styles.css**: 所有样式和视觉效果
- **app.js**: 所有业务逻辑和交互功能

## 🚀 使用方法

服务器内置了 HTTP 服务器

1. 启动服务器：
   ```bash
   cd ballballu
   cargo run -p server
   ```

   服务器会启动两个服务：
   - **WebSocket 服务器**: `0.0.0.0:8000` (游戏通信)
   - **HTTP 服务器**: `0.0.0.0:8080` (静态文件服务)

2. 在浏览器访问：
   ```
   http://localhost:8080/test.html
   ```
   或者直接访问根路径：
   ```
   http://localhost:8080/
   ```

## 🎮 功能说明

### 连接控制
- **Server URL**: 配置 WebSocket 服务器地址（默认：ws://127.0.0.1:8000）
- **Connect**: 连接到服务器
- **Disconnect**: 断开连接

### 游戏控制
- **Player Name**: 设置玩家名称
- **Join Game**: 加入游戏
- **Movement**: 使用箭头按钮或键盘 WASD 移动
- **Stop**: 停止移动（或按空格键）
- **Quit Game**: 退出游戏

### 显示信息
- **Current Tick**: 当前游戏 tick 数
- **Players Online**: 在线玩家数量
- **Your Score**: 您的分数
- **Dots Remaining**: 地图上剩余的 dot 数量
- **Players List**: 所有在线玩家列表（您的玩家会高亮显示）
- **Message Log**: WebSocket 消息日志（发送/接收的所有消息）

## ⌨️ 键盘快捷键

- `W` / `↑`: 向上移动
- `S` / `↓`: 向下移动
- `A` / `←`: 向左移动
- `D` / `→`: 向右移动
- `Space`: 停止移动

## 📝 测试流程示例

1. 打开 `test.html` 文件
2. 点击 **Connect** 按钮连接到服务器
3. 输入玩家名称（可选）
4. 点击 **Join Game** 加入游戏
5. 使用键盘或按钮控制移动
6. 观察右侧面板的游戏状态更新
7. 打开多个浏览器窗口测试多玩家交互

## 🐛 故障排除

### 连接失败
- 确保服务器正在运行（`cargo run -p server`）
- 检查服务器地址是否正确（默认：ws://127.0.0.1:8000）
- 查看浏览器控制台是否有错误信息

### 没有收到更新
- 确保已经点击 **Join Game**
- 检查 Message Log 中是否有 StateUpdate 消息
- 服务器应该每 50ms 发送一次更新

### 无法移动
- 确保已经加入游戏（Join Game）
- 检查 Message Log 确认输入消息已发送
- 服务器日志应该显示 "Player X input: dx=..., dy=..."

## 📚 消息格式参考

### 客户端 → 服务器

**加入游戏**:
```json
{ "Join": { "name": "PlayerName" } }
```

**移动输入**:
```json
{
  "Input": {
    "input": {
      "dx": 1.0,
      "dy": 0.0,
      "sequence_number": 1
    }
  }
}
```

**退出游戏**:
```json
{ "Quit": null }
```

### 服务器 → 客户端

**状态更新**:
```json
{
  "StateUpdate": {
    "snapshot": {
      "tick": 123,
      "players": [...],
      "dots": [...],
      "constants": {...}
    }
  }
}
```

## 🎨 界面特性

- 响应式设计
- 实时状态更新
- 彩色消息日志
- 玩家列表高亮显示当前玩家
- 统计数据卡片
- 键盘和鼠标双重控制

