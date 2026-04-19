# 线上发布

本页是 SDKWork API Router 的标准 GitHub 托管发布 runbook。

当你要发布正式 GitHub Release、准备仓库变量与 Secrets、接入 desktop 签名，或在 workflow 完成后校验线上发布结果时，请使用本页。

对外公开的发布契约会刻意收窄：

- 可安装正式产品：
  - `sdkwork-api-router-product-server`
  - `sdkwork-router-portal-desktop`
- 对外发布的元数据：
  - `release-catalog.json`

治理证据、telemetry 导出、release-window 快照、sync audit 以及 smoke 证据都保留为 workflow artifacts 和 attestations，不作为公开下载资产。

## 支持的发布触发方式

仓库当前支持两种正式触发方式：

- tag push：
  - 任意匹配 `release-*` 的 tag
- `workflow_dispatch`：
  - `release_tag` 必填
  - `git_ref` 选填，默认回落到 `refs/tags/<release_tag>`

推荐 tag 示例：

- `release-2026-04-19`
- `release-v1.0.0`

如果你希望发布具备可复现和可审计能力，应当先让目标提交通过本地受治理验证，再创建正式 tag。

## 推荐的本地预检

在 push release tag 或执行 `workflow_dispatch` 之前，请在目标提交上先跑仓库侧的受治理验证：

```bash
./bin/build.sh --verify-release
node --test scripts/release/tests/release-workflow.test.mjs
node --test scripts/release-governance-workflow.test.mjs
node --test scripts/product-verification-workflow.test.mjs
node --test scripts/rust-verification-workflow.test.mjs
node --test scripts/check-router-docs-safety.test.mjs
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\bin\build.ps1 -VerifyRelease
node --test scripts/release/tests/release-workflow.test.mjs
node --test scripts/release-governance-workflow.test.mjs
node --test scripts/product-verification-workflow.test.mjs
node --test scripts/rust-verification-workflow.test.mjs
node --test scripts/check-router-docs-safety.test.mjs
```

这会先验证正式本地构建、打包后运行时 smoke、docs 治理，以及 release、release-governance、product-verification、rust-verification 这些关键 workflow 的合同，避免 GitHub runner 在明显不满足发布契约时继续消耗时间。

## 必备仓库变量

workflow 会在验证和打包之前物化被引用的外部 release 依赖。请在仓库变量中明确这些 ref：

- `SDKWORK_CORE_GIT_REF`
- `SDKWORK_UI_GIT_REF`
- `SDKWORK_APPBASE_GIT_REF`
- `SDKWORK_CRAW_CHAT_SDK_GIT_REF`

如果你希望追求更强的发布可追溯性，推荐使用不可变 commit SHA；如果变量未定义，workflow 会默认回落到 `main`。

## 治理输入与覆盖项

治理流水线既可以消费仓库内提交的受治理制品，也可以接收控制平面通过仓库变量下发的显式覆盖。

主要治理覆盖变量：

- `SDKWORK_RELEASE_WINDOW_SNAPSHOT_JSON`
- `SDKWORK_RELEASE_SYNC_AUDIT_JSON`
- `SDKWORK_RELEASE_TELEMETRY_EXPORT_JSON`

Telemetry 补充变量：

- `SDKWORK_RELEASE_TELEMETRY_GATEWAY_PROMETHEUS_TEXT`
- `SDKWORK_RELEASE_TELEMETRY_ADMIN_PROMETHEUS_TEXT`
- `SDKWORK_RELEASE_TELEMETRY_PORTAL_PROMETHEUS_TEXT`
- `SDKWORK_RELEASE_TELEMETRY_SUPPLEMENTAL_TARGETS_JSON`
- `SDKWORK_RELEASE_TELEMETRY_GENERATED_AT`
- `SDKWORK_RELEASE_TELEMETRY_SOURCE_KIND`
- `SDKWORK_RELEASE_TELEMETRY_SOURCE_PROVENANCE`
- `SDKWORK_RELEASE_TELEMETRY_SOURCE_FRESHNESS_MINUTES`

Attestation 策略：

- `SDKWORK_RELEASE_ARTIFACT_ATTESTATIONS_ENABLED`
  - 私有仓库如需产出 artifact attestations，请显式设置为 `true`
  - 公有仓库按当前 workflow 契约会默认生成 attestation

如果没有提供显式 JSON 覆盖，workflow 会从仓库内既有的受治理路径加载对应制品并继续物化。

## Desktop 签名配置

现在 `release.yml` 中的 `Run portal desktop signing hook` 已经直接消费 GitHub 仓库变量与 Secrets，这意味着线上发布不再需要手工改 workflow 才能接入签名。

Fail-closed 策略：

- `SDKWORK_RELEASE_DESKTOP_SIGNING_REQUIRED`
  - 设为 `true` 时，若未配置签名 hook，desktop 发布应直接失败

签名 hook 来源：

- `SDKWORK_RELEASE_DESKTOP_WINDOWS_SIGN_HOOK`
- `SDKWORK_RELEASE_DESKTOP_LINUX_SIGN_HOOK`
- `SDKWORK_RELEASE_DESKTOP_MACOS_SIGN_HOOK`
- `SDKWORK_RELEASE_DESKTOP_SIGN_HOOK`

解析规则：

- 平台专属 hook 优先于通用 hook
- workflow 会先读取 `secrets.*`，再回落到 `vars.*`
- 涉及签名命令、证书、token 等敏感信息时，应优先放在 GitHub Secrets 中

hook 占位符：

- `{app}`
- `{platform}`
- `{arch}`
- `{target}`
- `{file}`
- `{evidence}`

仓库侧契约负责发现 installer、执行 hook，并把证据写入：

- `artifacts/release-governance/desktop-release-signing-<platform>-<arch>.json`

具体调用平台签名或 notarization 工具链的责任仍由 hook 命令本身承担。

## 触发线上发布

### 方式一：Push 正式 Tag

```bash
git tag release-2026-04-19
git push origin release-2026-04-19
```

这是更推荐的正式路径，因为 tag 本身就是发布身份，同时也是默认构建 ref。

### 方式二：执行 `workflow_dispatch`

适用场景：

- tag 已经存在，需要重试 workflow
- 需要显式指定 `git_ref` 重新构建
- 需要从受控 tag 或受保护 ref 做发布演练

必填输入：

- `release_tag`

选填输入：

- `git_ref`

如果 `git_ref` 为空，workflow 会默认构建 `refs/tags/<release_tag>`。

## Workflow 会产出什么

线上 workflow 当前分成几个主要阶段：

- `rust-dependency-audit`
  - 针对精确 release ref 做 Rust 依赖审计
- `product-verification`
  - 安装产品工作区依赖、构建 docs site，并执行仓库自带的 release 产品校验
- `governance-release`
  - 物化治理证据并上传 governance workflow artifacts
- `native-release`
  - 为各平台与架构构建正式 server 与 portal desktop 产品
- `publish`
  - 下载正式资产、重新生成 `release-catalog.json`，并创建 GitHub Release

## 发布后的校验

workflow 成功后，至少要做两层校验。

### GitHub Release 资产校验

确认正式 GitHub Release 中只包含：

- `sdkwork-api-router-product-server-*.tar.gz`
- `sdkwork-api-router-product-server-*.tar.gz.sha256.txt`
- `sdkwork-api-router-product-server-*.manifest.json`
- `sdkwork-router-portal-desktop-*`
- `release-catalog.json`

同时确认它不包含：

- governance bundle
- release-window 快照
- telemetry 导出
- sync audit
- 原始 Tauri bundle 树
- 独立 web 发布资产

### Workflow Artifacts 与 Attestations 校验

确认 workflow run 中仍然保留治理和 smoke 证据：

- `release-governance-bundle`
- `release-governance-window-snapshot`
- `release-governance-sync-audit`
- `release-governance-telemetry-export`
- `release-governance-telemetry-snapshot`
- `release-governance-slo-evidence`
- 各平台的 desktop signing evidence
- installed-runtime smoke evidence
- Linux Docker Compose smoke evidence
- Linux Helm render smoke evidence

如果仓库启用了 artifact attestations，还应确认这些对象存在 attestation：

- 治理证据
- desktop signing evidence
- smoke evidence
- packaged native release assets
- `release-catalog.json`

请在已发布 tag 或对应提交的仓库检出上运行仓库自带的 attestation 校验命令，而不是只依赖 GitHub 页面人工检查。该命令要求 `gh` 已在 `PATH` 中：

```bash
node scripts/release/verify-release-attestations.mjs --format text --repo Sdkwork-Cloud/sdkwork-api-router
```

```powershell
node scripts/release/verify-release-attestations.mjs --format text --repo Sdkwork-Cloud/sdkwork-api-router
```

如需把校验结果沉淀到自动化流程或归档记录中，可改用 `--format json`。

### Catalog 校验

检查 `release-catalog.json` 时，至少确认：

- 存在 `generatedAt`
- 只包含正式 product id
- 每个 variant 都包含 `variantKind`
- 每个 variant 都包含 `primaryFileSizeBytes`
- 每个 variant 都包含 `checksumAlgorithm`
- 每个 variant 指向的都是规范化后的正式发布文件名

## 相关文档

- [发布构建](/zh/getting-started/release-builds)
- [生产部署](/zh/getting-started/production-deployment)
- [安装布局](/zh/operations/install-layout)
- [服务管理](/zh/operations/service-management)
