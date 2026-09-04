# VibeShare

在电脑上 Vibe 出网页后，如果想在手机上查看，通常还要部署、等待构建、上传，再打开链接。发现问题后又要重复一次。

After you vibe a webpage on your Mac, checking it on a phone usually means deploying, waiting on a build, uploading, then opening a link. Find a problem, and you do it again.

VibeShare 让你跳过这套流程：电脑负责 Vibe 和修改，手机负责真实预览，VibeShare 负责把两者连接起来。

VibeShare skips that loop. The Mac vibes and edits. The phone shows the real result. VibeShare connects the two.

正在 Vibe 网页？不用部署、不用上传，手机扫码即可预览。文件一有变动，访问端自动同步。

Vibing a webpage? Skip deployment and cloud uploads. Scan the QR code to preview it on your phone, with live updates as you edit.

macOS 桌面应用，基于 **Tauri 2 + Rust + SvelteKit**。  
A macOS desktop app built with **Tauri 2 + Rust + SvelteKit**.

它适合同一局域网内的即时预览和临时分享，不替代正式部署。网页准备好上线时，再交给服务器或静态托管平台。

Built for instant preview and temporary sharing on the same local network — not a replacement for production hosting. Deploy to a server or static host when the page is ready to ship.

## 产品定位 / Positioning

- 本地即时预览 / 临时分享：VibeShare  
  Instant local preview / temporary sharing: VibeShare
- 正式生产发布：服务器或静态托管平台  
  Production release: a server or static host

## 传统流程与解决方式 / The loop it removes

传统流程 / The usual loop:

1. 在电脑上 Vibe 出一个网页  
   Vibe a page on the Mac
2. 部署到服务器或静态托管平台  
   Deploy to a server or static host
3. 等待构建、上传和发布  
   Wait on the build, upload, and publish
4. 手机上打开链接查看  
   Open the link on a phone
5. 发现问题后回到电脑修改  
   Find a problem, go back to the Mac
6. 再次部署，重复整个流程  
   Deploy again

VibeShare：连上同一个 Wi-Fi，扫码即可预览；文件一有变动，访问端自动同步。

With VibeShare: join the same Wi-Fi, scan to preview, and visitor pages refresh themselves as you edit.

## 怎么用 / How it works

`Vibe 网页 → 打开 VibeShare → 手机扫码 → 修改后实时查看`  
`Vibe → Share → Preview → Iterate`

1. **Vibe 网页 / Vibe** — 在电脑上生成或修改当前页面。 Generate or edit the page on your Mac.
2. **打开 VibeShare / Share** — 选文件夹或 HTML，扫码并输入临时访问码。 Pick a folder or HTML file, then scan the QR and enter a temporary code.
3. **手机扫码 / Preview** — 同一 Wi-Fi 的手机、平板或其他电脑立刻打开。 Phones, tablets, or other computers on the same Wi-Fi open it immediately.
4. **修改后实时查看 / Iterate** — 电脑上继续改，访问端自动刷新，不用再部署一遍。 Keep editing on the Mac. Visitors refresh themselves — no redeploy.

## 使用场景 / Use cases

- 电脑上改了网页，手机端立即检查，不需要反复部署测试环境  
  Edit on the Mac, check on a real phone — no staging environment to redeploy
- 客户、同事扫码即可看到当前版本，不需要发压缩包或等构建  
  Clients and teammates scan the current version — no zip file, no waiting on a build
- 作品还没上线，也能先给人看本地作品集、Demo 和原型  
  Show a local portfolio, demo, or prototype before it ships
- 展会、活动和门店现场，连上同一网络就能打开页面  
  Open an event or in-store page once devices are on the same network
- 课堂上立刻看到老师电脑上的示例，学生不用装开发环境  
  Students see the example from the teacher’s Mac without installing a dev setup
- 未公开的页面只在局域网里临时看，不放到公网  
  Keep unreleased pages on the local network instead of giving them a public URL

## 核心卖点 / Why it works

- **无需部署 / No deploy** — 跳过构建、上传和发布，直接分享本地文件。 Skip the build, upload, and publish step. Share the local files.
- **访问码保护 / Access code** — 临时展示给同事、客户或朋友，不公开暴露。 Show it to teammates, clients, or friends without making it public.
- **实时同步 / Live updates** — 电脑上继续修改，手机端自动刷新查看。 Keep editing on the Mac. The phone refreshes itself.

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

## 使用方式 / Using it

1. 打开 VibeShare，选择网站目录或 HTML 文件。  
   Open VibeShare and choose a site folder or HTML file.
2. 开始共享，扫码并输入临时访问码。  
   Start sharing, then scan the QR and enter a temporary access code.
3. 同一 Wi-Fi 的设备扫码打开。  
   Devices on the same Wi-Fi scan to open it.
4. 继续在电脑上修改，访问端自动刷新。  
   Keep editing on the Mac; visitors refresh themselves.

安装包见 [GitHub Releases](https://github.com/Heiuswork/VibeShare/releases)。  
Installers: [GitHub Releases](https://github.com/Heiuswork/VibeShare/releases).

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

## 打开方式 / Open on macOS

未签名的应用在别人电脑上打开，系统会提示「无法验证开发者」。让对方打开 **系统设置 → 隐私与安全性**，点「仍要打开」。GitHub Releases 发安装包时用附件，不要把 `src-tauri/target` 提交进仓库。

Unsigned apps trigger “Apple cannot verify the developer.” Ask the other person to open **System Settings → Privacy & Security** and click **Open Anyway**. Attach installers to GitHub Releases; do not commit `src-tauri/target`.

## 仓库说明 / Layout

| 目录 / Path | 内容 / Contents |
| --- | --- |
| `src/` | Svelte 界面 / Svelte UI |
| `src-tauri/` | Rust 核心、打包配置 / Rust core and bundling |
| `static/` | 静态资源 / Static assets |
| `docs/` | 产品宣传页 / Product site |

## License

见 / See [LICENSE](LICENSE).
