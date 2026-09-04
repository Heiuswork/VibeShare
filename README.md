# VibeShare

把电脑上刚 Vibe 出来的 HTML / 静态站点，立刻分享给同一局域网里的手机、平板或其他电脑。访问者不用装应用，扫码或打开链接就能看。文件只在本机提供，不上传云端。

Share HTML or a static site from your Mac to phones, tablets, and other computers on the same local network. Visitors don’t install anything — they scan a QR code or open a link. Files stay on your machine and never go to the cloud.

macOS 桌面应用，基于 **Tauri 2 + Rust + SvelteKit**。  
A macOS desktop app built with **Tauri 2 + Rust + SvelteKit**.

## 能做什么 / Features

- 选文件夹或 HTML 文件，本机预览，局域网扫码打开  
  Pick a folder or HTML file, preview locally, and open it on the LAN by scanning a QR code
- 临时访问码，不是谁扫都能进  
  Temporary access code — not everyone who scans gets in
- 改文件后访问端自动刷新，不用手动重载  
  Visitor pages refresh themselves when files change
- 可转发本机 Vite / Next 等开发服务器，动态路由和热更新一起过去  
  Proxy a local Vite / Next dev server so dynamic routes and HMR come along
- 在线设备列表、屏蔽本次访问、拉入黑名单  
  See who’s online, block this share, or blacklist a device
- 多网卡时指定分享用的那一张  
  Pick which network adapter to share on
- 静态资源诊断：缺文件、写死 `localhost` 的引用  
  Diagnose missing assets and hardcoded `localhost` URLs
- 中文 / English 界面  
  Chinese / English UI

## 环境 / Requirements

- macOS 13+
- Node 22+
- Rust stable
- Xcode Command Line Tools

## 开发 / Development

```bash
cd /path/to/VibeShare
npm install
npm run dev:app
```

只跑前端网页 / Frontend only:

```bash
npm run dev
```

## 打包 / Build

`.app`：

```bash
npm run build:mac
```

产物 / Output: `src-tauri/target/release/bundle/macos/VibeShare.app`

`.dmg`：

```bash
npx tauri build --bundles dmg
```

产物 / Output: `src-tauri/target/release/bundle/macos/`

未签名的应用在别人电脑上打开，系统会提示「无法验证开发者」。让对方打开 **系统设置 → 隐私与安全性**，点「仍要打开」。GitHub Releases 发安装包时用附件，不要把 `src-tauri/target` 提交进仓库。

Unsigned apps trigger “Apple cannot verify the developer.” Ask the other person to open **System Settings → Privacy & Security** and click **Open Anyway**. Attach installers to GitHub Releases; do not commit `src-tauri/target`.

## 仓库说明 / Layout

| 目录 / Path | 内容 / Contents |
| --- | --- |
| `src/` | Svelte 界面 / Svelte UI |
| `src-tauri/` | Rust 核心、打包配置 / Rust core and bundling |
| `static/` | 静态资源 / Static assets |

## License

见 / See [LICENSE](LICENSE).
