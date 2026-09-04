# VibeShare

把电脑上刚 Vibe 出来的 HTML / 静态站点，立刻分享给同一局域网里的手机、平板或其他电脑。访问者不用装应用，扫码或打开链接就能看。文件只在本机提供，不上传云端。

macOS 桌面应用，基于 **Tauri 2 + Rust + SvelteKit**。

## 能做什么

- 选文件夹或 HTML 文件，本机预览，局域网扫码打开
- 临时访问码，不是谁扫都能进
- 改文件后访问端自动刷新，不用手动重载
- 可转发本机 Vite / Next 等开发服务器，动态路由和热更新一起过去
- 在线设备列表、屏蔽本次访问、拉入黑名单
- 多网卡时指定分享用的那一张
- 静态资源诊断：缺文件、写死 `localhost` 的引用
- 中文 / English 界面

## 环境

- macOS 13+
- Node 22+
- Rust stable
- Xcode Command Line Tools

## 开发

```bash
cd /path/to/VibeShare
npm install
npm run dev:app
```

只跑前端网页：

```bash
npm run dev
```

## 打包

`.app`：

```bash
npm run build:mac
```

产物：`src-tauri/target/release/bundle/macos/VibeShare.app`

`.dmg`：

```bash
npx tauri build --bundles dmg
```

产物：`src-tauri/target/release/bundle/macos/`

未签名的应用在别人电脑上打开，系统会提示「无法验证开发者」。让对方打开 **系统设置 → 隐私与安全性**，点「仍要打开」。GitHub Releases 发安装包时用附件，不要把 `src-tauri/target` 提交进仓库。

## 仓库说明

| 目录 | 内容 |
| --- | --- |
| `src/` | Svelte 界面 |
| `src-tauri/` | Rust 核心、打包配置 |
| `static/` | 静态资源 |

## License

见 [LICENSE](LICENSE)。
