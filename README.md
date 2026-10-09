# QuickNotion

QuickNotion 是一款基于 Tauri 2、Rust、Svelte 5 和 TypeScript 的轻量级 Windows 桌面便签应用。

## 安装说明

### 系统要求

- Windows 10 或 Windows 11。
- Windows x64 系统。
- 安装包运行后不需要登录或联网账号。

### 推荐安装方式

1. 双击 QuickNotion_0.1.0_x64-setup.exe。
2. 按照安装向导完成安装。
3. 安装完成后启动 QuickNotion。
4. 应用启动后默认只显示在 Windows 系统托盘，不会显示在底部任务栏。
5. 左键点击托盘图标即可显示或隐藏便签窗口。

### MSI 安装方式

如果系统环境或部署工具要求使用 MSI，可以双击 QuickNotion_0.1.0_x64_en-US.msi 完成安装。

### 使用说明

- 拖动顶部标题栏可以移动窗口。
- 点击加号可以新建文件夹或笔记。
- 点击图钉图标可以切换窗口置顶状态。
- 点击窗口右上角 X 只会隐藏窗口，应用仍会保留在系统托盘。
- 左键点击托盘图标可以重新显示窗口。
- 右键点击托盘图标只显示“退出应用”。
- 退出应用前会结束后台进程。

### 卸载方式

可以通过 Windows“设置 → 应用 → 已安装的应用”卸载 QuickNotion，也可以使用安装目录中的卸载程序。

### 数据说明

当前版本的便签数据保存在应用本地，不需要云端账号。卸载应用前如需保留数据，请不要手动删除应用数据目录。

## 开发命令

安装依赖：

npm install

启动开发模式：

npm run tauri dev

生成 Windows 安装包：

npm run tauri build

