# 线上发布

本页是 SDKWork API Router 的标准 GitHub 托管发布 runbook。

当你要发布正式 GitHub Release、准备仓库变量与 Secrets、接入 desktop 签名，或在 workflow 完成后校验线上发布结果时，请使用本页。

对外公开的发布契约会刻意收窄：

- 可安装正式产品：
  - `sdkwork-api-router-product-server`
  - `sdkwork-router-portal-desktop`
- 对外发布的元数据：
  - `release-catalog.json`

治理证据、telemetry 导出、release-window 快照、sync audit、third-party SBOM / notices 制品以及 smoke 证据都保留为 workflow artifacts 和 attestations，不作为公开下载资产。
Linux server 镜像会单独发布到 GHCR：`ghcr.io/<owner>/sdkwork-api-router:<release-tag>`，不会作为 GitHub Release 附件重复上传。
每个 Linux 架构的 GHCR 发布元数据也会作为 workflow 证据保留在 `artifacts/release-governance/ghcr-image-publish-<platform>-<arch>.json`，方便运维直接审计最终 `imageRef` 与 digest，而不需要回溯 runner 日志。
最终多架构 GHCR manifest 的发布也会在 `artifacts/release-governance/ghcr-image-manifest-publish.json` 中保留 workflow 证据，记录 release tag 对应的 `targetImageRef` 与 manifest digest。
这两个 GHCR JSON 文件本身就是机器可读的运维契约：

- `ghcr-image-publish-<platform>-<arch>.json` 必须包含 `version`、`type`（`sdkwork-ghcr-image-publish`）、`generatedAt`、`releaseTag`、`platform`、`arch`、`bundlePath`、`imageRepository`、`imageTag`、`imageRef`、`digest`。
- `ghcr-image-manifest-publish.json` 必须包含 `version`、`type`（`sdkwork-ghcr-image-manifest-publish`）、`generatedAt`、`releaseTag`、`imageRepository`、`targetImageTag`、`targetImageRef`、`sourceImageRefs`、`digest`、`manifestMediaType`、`platformCount`。

组装后的 `release-governance-bundle` workflow artifact 也会作为整棵 payload 树单独生成 provenance attestation，确保可下载的治理打包与单独的 governed JSON 证据具备同等级别的可审计性。

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
- `SDKWORK_IM_SDK_GIT_REF`

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
治理阶段还会直接生成以下第三方治理制品：

- `docs/release/third-party-sbom-latest.spdx.json`
- `docs/release/third-party-notices-latest.json`

这两份文件来自 Rust 依赖图以及 admin / portal 工作区在 frozen install 之后的 `node_modules/` 清单。

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

把该 desktop signing evidence 视为机器可读的运维合同：

- `desktop-release-signing-<platform>-<arch>.json` 会输出 `version`、`type`（`sdkwork-desktop-release-signing`）、`appId`、`platform`、`arch`、`targetTriple`、`required`、`status`、`hook`、`bundleFiles`、`evidencePath` 与 `commandCount`
- `hook` 会记录最终解析得到的 `kind` 以及命中的配置 `envVar`
- `status` 只会是 `skipped`、`signed` 或 `failed`
- 如果签名失败，还会附带 `failure.message`，便于自动化归档精确的签名失败原因

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
  - 安装受治理的 admin / portal 工作区依赖，物化治理证据并上传 governance workflow artifacts
  - 输出 `third-party-sbom-latest.spdx.json` 与 `third-party-notices-latest.json`
- `native-release`
  - 为各平台与架构构建正式 server 与 portal desktop 产品
  - 将 Linux OCI 镜像按架构发布到 GHCR：`:<release-tag>-linux-x64` 与 `:<release-tag>-linux-arm64`
- `publish`
  - 下载正式资产、重新生成 `release-catalog.json`、创建 GitHub Release，并组装多架构 GHCR manifest 标签 `ghcr.io/<owner>/sdkwork-api-router:<release-tag>`

## 发布后的校验

workflow 成功后，至少要做两层校验。

### GitHub Release 资产校验

确认正式 GitHub Release 中只包含：

- `sdkwork-api-router-product-server-*.tar.gz`
- `sdkwork-api-router-product-server-*.tar.gz.sha256.txt`
- `sdkwork-api-router-product-server-*.manifest.json`
- `sdkwork-router-portal-desktop-*`
- `release-catalog.json`

在安装任何下载资产之前，都应先使用同目录下的 `.sha256.txt` 文件完成校验：

```bash
sha256sum -c sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt
sha256sum -c sdkwork-router-portal-desktop-linux-x64.AppImage.sha256.txt
```

```powershell
$expected = (Get-Content .\sdkwork-api-router-product-server-linux-x64.tar.gz.sha256.txt).Split()[0]
$actual = (Get-FileHash -Algorithm SHA256 .\sdkwork-api-router-product-server-linux-x64.tar.gz).Hash.ToLowerInvariant()
if ($actual -ne $expected.ToLowerInvariant()) { throw 'checksum mismatch' }
```

同时应把对外发布的 `release-catalog.json` 视为正式 SKU 集合的 machine-readable release index：

- 顶层必须包含 `version`、`type`（`sdkwork-release-catalog`）、`releaseTag`、`generatedAt`、`productCount`、`variantCount` 与 `products`
- 每个 `products[]` 条目都要记录正式 `productId` 以及对应的 `variants[]`
- 每个 `variants[]` 条目都要包含 `platform`、`arch`、`outputDirectory`、`variantKind`、`primaryFile`、`primaryFileSizeBytes`、`checksumFile`、`checksumAlgorithm`、`manifestFile`、`sha256`，以及解析后的外部 `manifest`

对外发布的正式资产 manifest 也应当被视为 machine-readable install contract：

- `sdkwork-api-router-product-server-<platform>-<arch>.manifest.json` 必须输出 `type`（`product-server-archive`）、`productId`、`platform`、`arch`、`target`、`releaseVersion`、`archiveFile`、`checksumFile`、`embeddedManifestFile`、`installers`、`services`、`sites`、`bootstrapDataRoots` 与 `deploymentAssetRoots`
- `sdkwork-router-portal-desktop-<platform>-<arch>.manifest.json` 必须输出 `type`（`portal-desktop-installer`）、`productId`、`appId`、`platform`、`arch`、`target`、`artifactKind`、`installerFile`、`checksumFile`、`sourceBundlePath` 与 `embeddedRuntime`

同时确认它不包含：

- governance bundle
- release-window 快照
- telemetry 导出
- sync audit
- 原始 Tauri bundle 树
- 独立 web 发布资产

### GHCR 镜像校验

确认 GHCR 中可见以下标签：

- `ghcr.io/<owner>/sdkwork-api-router:<release-tag>`
- `ghcr.io/<owner>/sdkwork-api-router:<release-tag>-linux-x64`
- `ghcr.io/<owner>/sdkwork-api-router:<release-tag>-linux-arm64`

同时确认 workflow artifacts 中还保留了 GHCR 发布元数据文件，例如 `ghcr-image-publish-linux-x64.json` 与 `ghcr-image-publish-linux-arm64.json`，其中会记录最终 `imageRef` 和 pushed digest。
同时确认 workflow artifacts 中还保留了 `ghcr-image-manifest-publish.json`，其中会记录多架构 release tag 镜像的 `targetImageRef` 与最终 manifest digest。
发布后校验时，应把这些 JSON 文件当成最终事实来源，而不是只看 GitHub 页面：

- 每个 `sdkwork-ghcr-image-publish` 记录都应能追溯到实际打包产物 `bundlePath`，并明确 `imageRepository`、`imageTag`、最终 `imageRef` 和发布后的 `digest`
- `sdkwork-ghcr-image-manifest-publish` 记录应明确汇总后的 `targetImageRef`、参与组装的 `sourceImageRefs`、解析得到的 `manifestMediaType`，以及最终 `platformCount`

### Workflow Artifacts 与 Attestations 校验

确认 workflow run 中仍然保留治理和 smoke 证据：

- `release-governance-bundle`
- `release-governance-window-snapshot`
- `release-governance-sync-audit`
- `release-governance-telemetry-export`
- `release-governance-telemetry-snapshot`
- `release-governance-slo-evidence`
- `release-governance-third-party-sbom`
- `release-governance-third-party-notices`
- 各平台的 desktop signing evidence
- installed-runtime smoke evidence
- Linux Docker Compose smoke evidence
- Linux Helm render smoke evidence
- 各 Linux 架构的 GHCR 发布元数据
- GHCR 多架构 manifest 发布元数据

可下载的 `release-governance-bundle` 不是普通压缩包，而是机器可读的恢复契约：

- 其中必须包含 `release-governance-bundle-manifest.json`
- 该 manifest 会声明 `version`、`generatedAt`、`bundleEntryCount` 与 `artifacts`
- 每个 `artifacts[]` 条目都包含 `id`、`relativePath` 与 `sourceRelativePath`
- `restore.command` 是运维在回放已下载治理 bundle 时应使用的仓库自带恢复入口
- 执行 `node scripts/release/restore-release-governance-latest.mjs --artifact-dir <downloaded-dir>` 时，会返回包含 `repoRoot` 与 `restored` 的 JSON；其中每个 `restored[]` 条目都会记录 `id`、`sourcePath`、`outputPath` 与 `duplicateCount`

同样应把 smoke evidence 当作机器可读的运维合同：

- `unix-installed-runtime-smoke-<platform>-<arch>.json` 与 `windows-installed-runtime-smoke-<platform>-<arch>.json` 会输出 `generatedAt`、`ok`、`platform`、`arch`、`target`、`runtimeHome`、`evidencePath`、`backupBundlePath`、`backupRestoreVerified` 与 `healthUrls`
- installed-runtime smoke evidence 在启动、备份、恢复或健康检查失败时，还可能包含 `logs.stdout`、`logs.stderr` 与 `failure.message`
- `docker-compose-smoke-<platform>-<arch>.json` 会输出 `generatedAt`、`ok`、`platform`、`arch`、`executionMode`、`bundlePath`、`evidencePath`、`healthUrls`、`siteUrls`、`browserSmokeTargets` 与 `databaseAssertions`
- Docker Compose smoke evidence 还可能附带 `browserSmokeResults`、`composePs`、`logs.router`、`logs.postgres`、`diagnostics` 与 `failure.message`
- `helm-render-smoke-<platform>-<arch>.json` 会输出 `generatedAt`、`ok`、`platform`、`arch`、`bundlePath`、`evidencePath`、`renderedManifestPath` 与 `renderedKinds`
- Helm render smoke evidence 在模板渲染或 schema 校验失败时，还可能附带 `kubeconformSummary` 与 `failure.message`

同样应把 release governance 证据视为机器可读的运维合同：

- `release-window-snapshot-latest.json` 会输出 `generatedAt`、`source` 与 `snapshot`；其中 `snapshot` 记录 `latestReleaseTag`、`commitsSinceLatestRelease`、`workingTreeEntryCount` 与 `hasReleaseBaseline`
- `release-sync-audit-latest.json` 会输出 `generatedAt`、`source` 与 `summary`；其中 `summary` 包含 `releasable` 与 `reports`，每个 `reports[]` 条目都包含 `id`、`targetDir`、`expectedGitRoot`、`topLevel`、`remoteUrl`、`localHead`、`remoteHead`、`expectedRef`、`branch`、`upstream`、`ahead`、`behind`、`isDirty`、`reasons` 与 `releasable`
- `release-telemetry-export-latest.json` 会输出 `version`、`generatedAt`、`source`、`prometheus` 与 `supplemental`；其中 `source` 可包含 `kind`、`provenance` 与 `freshnessMinutes`，`prometheus` 必须包含 `gateway`、`admin`、`portal`，`supplemental.targets` 负责记录那些不是直接 Prometheus 推导出来的发布目标
- `release-telemetry-snapshot-latest.json` 会输出 `version`、`snapshotId`、`generatedAt`、`source` 与 `targets`；其中 `source` 可包含 `kind`、`exportKind`、`provenance`、`freshnessMinutes` 与 `supplementalTargetIds`
- `slo-governance-latest.json` 会输出 `version`、`baselineId`、`baselineDate`、`generatedAt` 与 `targets`

同样应把 third-party governance 制品视为机器可读的运维合同：

- `third-party-sbom-latest.spdx.json` 会以 `SPDX-2.3` 文档形式输出，包含 `spdxVersion`、`documentNamespace`、`creationInfo.created`、`documentDescribes`、`packages` 与 `relationships`
- `third-party-notices-latest.json` 会输出 `version`、`generatedAt`、`packageCount`、`cargoPackageCount`、`npmPackageCount`、`packages` 与 `noticeText`
- 每个 third-party notices `packages[]` 条目至少都会记录 `ecosystem`、`name`、`version`、`licenseDeclared`、`downloadLocation`、`sourcePath` 与 `noticeFiles`

如果仓库启用了 artifact attestations，还应确认这些对象存在 attestation：

- governance bundle payloads
- 治理证据，包括 `third-party-sbom-latest.spdx.json` 与 `third-party-notices-latest.json`
- desktop signing evidence
- smoke evidence
- GHCR 发布元数据
- GHCR 多架构 manifest 发布元数据
- packaged native release assets
- `release-catalog.json`

请在已发布 tag 或对应提交的仓库检出上运行仓库自带的 attestation 校验命令，而不是只依赖 GitHub 页面人工检查。该命令要求 `gh` 已在 `PATH` 中：

```bash
node scripts/release/verify-release-attestations.mjs --format text --repo Sdkwork-Cloud/sdkwork-api-router
```

```powershell
node scripts/release/verify-release-attestations.mjs --format text --repo Sdkwork-Cloud/sdkwork-api-router
```

该校验器会覆盖受治理 JSON 制品，包括 `third-party-sbom-latest.spdx.json` 与 `third-party-notices-latest.json`，`release-governance-bundle` 内容树、desktop signing evidence、installed-runtime smoke evidence、Linux Docker Compose 与 Helm smoke evidence、GHCR 发布元数据、GHCR 多架构 manifest 发布元数据、正式 `release-catalog.json`，以及打包后的 native release 资产树。

如需让自动化直接消费校验结果，请使用 JSON 输出：

```bash
node scripts/release/verify-release-attestations.mjs --format json --repo Sdkwork-Cloud/sdkwork-api-router
```

```powershell
node scripts/release/verify-release-attestations.mjs --format json --repo Sdkwork-Cloud/sdkwork-api-router
```

该 JSON 结果本身就是机器可读的 attestation verdict contract：

- 顶层字段包括 `ok`、`blocked`、`reason`、`repoSlug`、`verifiedCount`、`blockedCount`、`failedCount`、`verifiedIds`、`blockedIds`、`failingIds` 与 `reports`
- 每个 `reports[]` 条目都包含 `id`、`specId`、`description`、`ok`、`blocked`、`reason`、`relativeSubjectPath`、`expectedRelativePath`、`stdout`、`stderr` 与 `errorMessage`

如需把校验结果沉淀到自动化流程或归档记录中，应优先使用这份 `--format json` 输出。

### Catalog 校验

检查 `release-catalog.json` 时，至少确认：

- 顶层 `type` 仍然是 `sdkwork-release-catalog`
- 存在 `releaseTag`、`productCount`、`variantCount` 与 `products`
- 存在 `generatedAt`
- 只包含正式 product id
- 每个 variant 都包含 `outputDirectory`、`variantKind`、`primaryFile`、`checksumFile`、`manifestFile`、`sha256` 以及解析后的 `manifest`
- 每个 variant 都包含 `primaryFileSizeBytes`
- 每个 variant 都包含 `checksumAlgorithm`
- 每个 variant 指向的都是规范化后的正式发布文件名

## 相关文档

- [发布构建](/zh/getting-started/release-builds)
- [生产部署](/zh/getting-started/production-deployment)
- [安装布局](/zh/operations/install-layout)
- [服务管理](/zh/operations/service-management)
