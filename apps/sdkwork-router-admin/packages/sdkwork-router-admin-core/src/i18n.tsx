import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';

export type AdminLocale = 'en-US' | 'zh-CN';

type TranslationValues = Record<string, number | string>;

type AdminI18nContextValue = {
  formatCurrency: (value: number, fractionDigits?: number) => string;
  formatDateTime: (value?: number | null) => string;
  formatNumber: (value: number) => string;
  locale: AdminLocale;
  setLocale: (locale: AdminLocale) => void;
  t: (text: string, values?: TranslationValues) => string;
};

const ADMIN_I18N_STORAGE_KEY = 'sdkwork-router-admin.locale.v2';
const AdminI18nContext = createContext<AdminI18nContextValue | null>(null);

export const ADMIN_LOCALE_OPTIONS: Array<{ id: AdminLocale; label: string }> = [
  { id: 'en-US', label: 'English' },
  { id: 'zh-CN', label: 'Simplified Chinese' },
];

let activeAdminLocale: AdminLocale = 'en-US';

const ADMIN_ZH_EXPANDED_TRANSLATIONS: Record<string, string> = {
  'API key': 'API 密钥',
  'API key scope': 'API 密钥范围',
  Adapter: '适配器',
  'Adapter kind': '适配器类型',
  'Add model pricing': '新增模型定价',
  'Add pricing': '新增定价',
  'Add rule': '新增规则',
  'All keys': '全部密钥',
  'Apply setup': '应用配置',
  'Applying...': '应用中...',
  Attention: '需关注',
  'Authorization header': '授权请求头',
  Backend: '后端',
  'Bound channels': '绑定通道',
  'Cache read': '缓存读取',
  'Cache write': '缓存写入',
  Capabilities: '能力',
  Channel: '通道',
  'Channel bindings': '通道绑定',
  'Channel id': '通道 ID',
  'Channel name': '通道名称',
  'Channel profile': '通道资料',
  'Context window': '上下文窗口',
  'Copy API key': '复制 API 密钥',
  'Copy plaintext API key': '复制明文 API 密钥',
  'Create channel': '创建通道',
  'Create mapping': '创建映射',
  'Create model mapping': '创建模型映射',
  'Create provider': '创建供应商',
  'Create route provider': '创建路由供应商',
  Credential: '凭据',
  'Credential coverage is empty': '凭据覆盖为空',
  'Credential details': '凭据详情',
  'Credential inventory': '凭据清单',
  'Ctrl K': 'Ctrl K',
  'Currency code': '币种代码',
  Custom: '自定义',
  'Custom API key': '自定义 API 密钥',
  'Custom provider': '自定义供应商',
  'Custom route': '自定义路由',
  Description: '描述',
  'Edit API key': '编辑 API 密钥',
  'Edit channel': '编辑通道',
  'Edit channel publication': '编辑通道发布',
  'Edit model mapping': '编辑模型映射',
  'Edit model pricing': '编辑模型定价',
  'Edit provider': '编辑供应商',
  'Edit route provider': '编辑路由供应商',
  'Effective from': '生效开始',
  'Effective to': '生效结束',
  Enabled: '启用',
  'Expires at': '到期时间',
  Extension: '扩展',
  'Extension id': '扩展 ID',
  Extensions: '扩展数',
  'Gateway default': '网关默认',
  'Gateway endpoint': '网关端点',
  'Gateway overlay mapping': '网关覆盖映射',
  'Global instance': '全局实例',
  'Hashed key': '哈希密钥',
  Healthy: '健康',
  'Hotspot candidate': '热点候选',
  Inactive: '未启用',
  'Input price': '输入价格',
  Inspect: '查看',
  'Instance id': '实例 ID',
  'Instance inventory is empty': '实例清单为空',
  Key: '密钥',
  'Key lifecycle': '密钥生命周期',
  'Key metadata': '密钥元数据',
  'Key reference': '密钥引用',
  'Key scoped': '按密钥范围',
  Label: '标签',
  'Last used': '最后使用',
  Live: '正式环境',
  'Loading local instances...': '正在加载本地实例...',
  'Managed runtimes': '托管运行时',
  'Mapping name': '映射名称',
  'Mapping profile': '映射资料',
  Message: '消息',
  'Model id': '模型 ID',
  'Model mapping': '模型映射',
  Models: '模型',
  'New model': '新建模型',
  'New provider': '新建供应商',
  'No API keys match the current filter': '没有 API 密钥匹配当前筛选条件',
  'No channel publications yet': '暂无通道发布',
  'No channels match the search': '没有通道匹配搜索条件',
  'No credentials match the search': '没有凭据匹配搜索条件',
  'No mapping': '无映射',
  'No message recorded': '未记录消息',
  'No model mapping': '无模型映射',
  'No providers match the search': '没有供应商匹配搜索条件',
  'No route providers match the current filter': '没有路由供应商匹配当前筛选条件',
  'No runtimes match the current filters': '没有运行时匹配当前筛选条件',
  'No variants match the search': '没有变体匹配搜索条件',
  Observed: '观测时间',
  'Open ended': '长期有效',
  'Open workspace search': '打开工作区搜索',
  'OpenClaw instances': 'OpenClaw 实例',
  'Operational posture': '运行态势',
  'Output price': '输出价格',
  'Page {page} of {totalPages}': '第 {page} 页 / 共 {totalPages} 页',
  'Pinned provider': '固定供应商',
  'Plaintext not available': '明文不可用',
  'Price unit': '计价单位',
  'Pricing is empty': '定价为空',
  'Pricing row': '定价行',
  'Primary channel': '主通道',
  'Provider health': '供应商健康',
  'Provider healthy': '供应商健康',
  'Provider id': '供应商 ID',
  'Provider mode': '供应商模式',
  'Provider profile': '供应商资料',
  'Provider requires attention': '供应商需要关注',
  'Provider variants': '供应商变体',
  Providers: '供应商',
  Publication: '发布',
  Publish: '发布',
  'Publish model': '发布模型',
  'Publish model to channel': '将模型发布到通道',
  'Publish to channel': '发布到通道',
  'Published models': '已发布模型',
  'Quick setup': '快速配置',
  'Quota exhaustion detected': '检测到配额耗尽',
  'Refreshing workspace': '正在刷新工作区',
  Remove: '移除',
  'Request price': '请求价格',
  Revoke: '撤销',
  Revoked: '已撤销',
  'Rotate credential': '轮换凭据',
  'Route config': '路由配置',
  'Route mode': '路由模式',
  'Route overlay': '路由覆盖',
  'Route policy': '路由策略',
  'Route posture': '路由态势',
  'Rule builder': '规则构建器',
  'Rule {index}': '规则 {index}',
  Running: '运行中',
  Runtime: '运行时',
  'Runtime family': '运行时族',
  'Runtime healthy': '运行时健康',
  'Runtime reload failed': '运行时重载失败',
  'Runtime reload finished': '运行时重载完成',
  'Runtime requires attention': '运行时需要关注',
  'Save changes': '保存更改',
  'Save channel': '保存通道',
  'Save credential': '保存凭据',
  'Save mapping': '保存映射',
  'Save pricing': '保存定价',
  'Save provider': '保存供应商',
  'Save publication': '保存发布',
  'Save route config': '保存路由配置',
  'Secret value': '密钥值',
  'Setup status': '配置状态',
  Source: '来源',
  'Source model': '源模型',
  Stopped: '已停止',
  Storage: '存储',
  Streaming: '流式',
  'System generated': '系统生成',
  Target: '目标',
  'Target model': '目标模型',
  'Targeted runtime reload': '定向运行时重载',
  Test: '测试环境',
  Usage: '使用',
  'Usage method': '使用方式',
  Variants: '变体',
  'Visible providers': '可见供应商',
  'Visible runtimes': '可见运行时',
  'Workspace default API key': '工作区默认 API 密钥',
  'Workspace scope': '工作区范围',
  Yes: '是',
  'optional extension id': '可选扩展 ID',
  'optional instance id': '可选实例 ID',
  '{count} API keys': '{count} 个 API 密钥',
  '{count} credentials': '{count} 个凭据',
  '{count} expiring soon': '{count} 个即将到期',
  '{count} pricing rows': '{count} 条定价记录',
  '{count} providers': '{count} 个供应商',
  'Applied setup and updated {count} environment target(s).': '已应用配置并更新 {count} 个环境目标。',
  'Applied setup and wrote {count} file(s).': '已应用配置并写入 {count} 个文件。',
  'Applied setup to {count} OpenClaw instance(s).': '已将配置应用到 {count} 个 OpenClaw 实例。',
  'Authorization: Bearer {token}': 'Authorization: Bearer {token}',
  'Capture the upstream endpoint and the public channel bindings used by the router.':
    '记录路由器所使用的上游端点与公开通道绑定。',
  'Capture upstream connectivity and channel bindings with the shared form primitives.':
    '使用共享表单组件记录上游连接与通道绑定。',
  'Channel model publications and their pricing rows.': '通道模型发布及其定价记录。',
  'Choose a source model and a target model for each translation row.': '为每一条转换规则选择源模型与目标模型。',
  'Choose the local instances that should receive the generated setup.': '选择应接收生成配置的本地实例。',
  'Current route provider mode for the selected key.': '当前所选密钥对应的路由供应商模式。',
  'Distinct extensions represented by the current slice.': '当前视图范围内涉及的扩展数量。',
  'Distinct status strings in the current slice.': '当前视图范围内不同状态字符串的数量。',
  'Encrypted provider credentials on record.': '已登记的加密供应商凭据数量。',
  'Expose the provider on one or more public API channels without leaving the dialog.':
    '无需离开当前对话框即可将供应商暴露到一个或多个公开 API 通道。',
  'Full runtime reload completed.': '已完成完整运行时重载。',
  'Gateway compatibility endpoint for OpenAI-style clients.': '面向 OpenAI 风格客户端的网关兼容端点。',
  'Issue a new API key, set its workspace scope, and define the initial route posture in one flow.':
    '在一个流程中签发新的 API 密钥、设置工作区范围并定义初始路由态势。',
  'Keep channel metadata focused and manage publications separately from the detail rail.':
    '保持通道元数据聚焦展示，并将发布管理与详情侧栏分离。',
  'Keys currently bound to the selected provider.': '当前绑定到所选供应商的密钥。',
  'Leave empty to let the gateway generate the plaintext.': '留空则由网关自动生成明文密钥。',
  'Map one client-facing model shape onto a target channel model.': '将一个面向客户端的模型形态映射到目标通道模型。',
  'Map one public-facing source model onto a target channel model.': '将一个对外公开的源模型映射到目标通道模型。',
  'Model mapping applied to this key, if any.': '应用于该密钥的模型映射（如有）。',
  'No OpenClaw instances were detected on this machine yet.': '当前机器尚未检测到 OpenClaw 实例。',
  'No encrypted credential records are currently assigned to this provider.': '当前没有加密凭据记录分配给该供应商。',
  'No provider health rows match the current filters': '没有供应商健康记录匹配当前筛选条件',
  'No provider pricing rows exist for the selected publication.': '当前所选发布项下没有供应商定价记录。',
  'Operational posture remains inspectable without leaving the active table context.':
    '无需离开当前表格上下文即可继续查看运行态势。',
  'Pick a client profile to view and apply the generated setup snippets.':
    '选择一个客户端配置，以查看并应用生成的配置片段。',
  'Pricing for {name}': '{name} 的定价',
  'Provider health snapshots with operator-facing status and latest message context.':
    '面向运营人员展示供应商健康快照、当前状态和最新消息上下文。',
  'Provider health stays visible alongside status and operator notes.':
    '供应商健康信息会与状态及运营备注一并展示。',
  'Provider-specific billing rows for the selected publication.': '所选发布项对应的供应商计费记录。',
  'Provider-specific pricing rows stay aligned with the shared catalog workbench.':
    '供应商专属定价记录与共享目录工作台保持一致。',
  'Provider-scoped variants known to the router.': '路由器已识别的供应商范围变体。',
  'Proxy providers bound into the catalog.': '已绑定到目录中的代理供应商。',
  'Public API channels currently pointing at this provider.': '当前指向该供应商的公开 API 通道。',
  'Public channels exposed by the router.': '由路由器暴露的公开通道。',
  'Publish a provider model into this channel to start exposing it to router consumers.':
    '将供应商模型发布到该通道后，即可对路由消费方开放。',
  'Publish a provider variant into a public channel using the shared form system.':
    '使用共享表单系统将供应商变体发布到公开通道。',
  'Rotate or create encrypted provider credentials without leaving the catalog workbench.':
    '无需离开目录工作台即可轮换或创建加密供应商凭据。',
  'Runtime family, extension, health, and current operator message in one dense table.':
    '在一张紧凑表格中展示运行时族、扩展、健康状态与当前运营消息。',
  'Runtime state remains inspectable with family, extension, instance, and message context.':
    '可结合运行时族、扩展、实例与消息上下文持续查看运行状态。',
  'Scope the reload to a specific extension or runtime instance when the full control-plane refresh is unnecessary.':
    '当无需完整刷新控制平面时，可将重载范围限定到特定扩展或运行时实例。',
  'Select the tenant, project, and environment that own the new key.': '选择拥有新密钥的租户、项目和环境。',
  'Selected key label or workspace fallback.': '所选密钥标签，若无则回退到工作区标识。',
  'Selected provider mode and mapping for this key.': '该密钥当前选择的供应商模式与映射。',
  'Traffic inspection keeps the active lens and filter window attached to the selected row.':
    '流量检查会保持当前视角与筛选窗口绑定到所选行。',
  'Try a broader query or add a new channel.': '尝试更宽泛的查询，或新增一个通道。',
  'Try a broader query or add a new credential.': '尝试更宽泛的查询，或新增一条凭据。',
  'Try a broader query or add a new provider.': '尝试更宽泛的查询，或新增一个供应商。',
  'Try a broader query or publish a new provider variant upstream.':
    '尝试更宽泛的查询，或向上游发布新的供应商变体。',
  'Try a broader query to inspect more provider health rows.': '尝试更宽泛的查询，以查看更多供应商健康记录。',
  'Try a broader query to inspect more runtime statuses.': '尝试更宽泛的查询，以查看更多运行时状态。',
  'Try a broader search or reset the channel and health filters.': '尝试更宽泛的搜索，或重置通道与健康筛选条件。',
  'Try a broader search query or create a new key.': '尝试更宽泛的搜索条件，或创建一个新密钥。',
  'Update display metadata without changing the current workbench selection.':
    '更新展示元数据，同时不改变当前工作台的选择状态。',
  'cURL smoke test': 'cURL 冒烟测试',
  No: '否',
  'Providers currently marked healthy.': '当前标记为健康的供应商数量。',
  'Providers requiring operator attention.': '当前需要运营关注的供应商数量。',
  'Providers visible in the current slice.': '当前范围内可见的供应商数量。',
  'Runtimes currently marked healthy.': '当前标记为健康的运行时数量。',
  'Runtimes currently running.': '当前正在运行的运行时数量。',
  'Runtimes visible in the current slice.': '当前范围内可见的运行时数量。',
  'SDKWork gateway default': 'SDKWork 网关默认',
  'Status variants': '状态种类',
  'Targeted reload completed for {scope}.': '已完成 {scope} 的定向重载。',
  'database envelope': '数据库信封加密',
  'local encrypted file': '本地加密文件',
  'os keyring': '系统钥匙串',
};

const ADMIN_ZH_COMPLETION_TRANSLATIONS: Record<string, string> = {
  '4102444800000': '4102444800000',
  '{count} attached': '已关联 {count} 个',
  '{count} disabled': '已停用 {count} 个',
  '{count} mappings': '{count} 条映射',
  '{count} project(s) currently sit at the quota ceiling and may require intervention.':
    '当前有 {count} 个项目达到配额上限，可能需要人工干预。',
  '{count} published models': '已发布 {count} 个模型',
  '{count} records': '{count} 条记录',
  '{count} tenants': '{count} 个租户',
  '{days} days left in the current window.': '当前窗口剩余 {days} 天。',
  '{days} days of runway remain.': '当前剩余可用期 {days} 天。',
  '{days} days overdue.': '已逾期 {days} 天。',
  '{days} days remain before campaign expiry.': '距离活动过期还剩 {days} 天。',
  '{lane} directory': '{lane} 目录',
  '{model} request': '{model} 请求',
  '{remaining} units available for redemptions.': '还有 {remaining} 个单位可用于兑换。',
  '{remaining} units remaining before depletion.': '在耗尽前还剩 {remaining} 个单位。',
  '{scope} {activeCount} active runtime(s) and {packageCount} loadable package(s) were discovered at {reloadedAt}.':
    '在 {reloadedAt} 发现 {scope} 下有 {activeCount} 个活跃运行时和 {packageCount} 个可加载包。',
  Accent: '强调色',
  'Accent preset': '强调色预设',
  active: '活跃',
  'Active users': '活跃用户',
  'Aggregate amount in the current slice.': '当前范围内的汇总金额。',
  'Aggregate booked amount.': '汇总已记账金额。',
  'Aggregate hotspot amount.': '热点项目汇总金额。',
  'Aggregate hotspot request volume.': '热点项目汇总请求量。',
  'Aggregate hotspot tokens.': '热点项目汇总令牌数。',
  'Aggregate tokens in the current slice.': '当前范围内的汇总令牌数。',
  'Aggregate units in the current slice.': '当前范围内的汇总用量单位。',
  'Aggregate used units.': '汇总已用单位。',
  'Aggregate visible amount.': '可见范围汇总金额。',
  'Aggregate visible request volume.': '可见范围汇总请求量。',
  'Already have an account?': '已有账号？',
  'API key lifecycle, route posture, and bootstrap workflows stay attached to the selected registry row.':
    'API 密钥生命周期、路由姿态和引导流程始终绑定在所选注册表行。',
  Appearance: '外观',
  'Appearance, navigation, and workspace sections now live in a real settings center instead of a standalone preferences panel.':
    '外观、导航和工作区设置现已集中到真正的设置中心，而不是独立的偏好面板。',
  Archive: '归档',
  'Back to login': '返回登录',
  'Billing posture stays aligned with the current quota policy and remaining units.':
    '计费姿态始终与当前配额策略和剩余单位保持一致。',
  'Billing summary': '计费汇总',
  'Booked amount': '已记账金额',
  'Bright shell with frosted content panes.': '明亮的壳层界面，配合毛玻璃内容面板。',
  'Broaden the query or widen the time window to inspect more routing decisions.':
    '请放宽查询条件或扩大时间窗口，以查看更多路由决策。',
  'Broaden the query or widen the time window to inspect more usage records.':
    '请放宽查询条件或扩大时间窗口，以查看更多用量记录。',
  'Broaden the selected API key or time range, or clear the text search.':
    '请放宽所选 API 密钥或时间范围，或清除文本搜索。',
  'Build one or more source-to-target translation rules without leaving the shared gateway workbench.':
    '无需离开共享网关工作台，即可构建一条或多条源到目标的转换规则。',
  'Campaign is disabled for new redemptions.': '当前活动已停用新的兑换操作。',
  'Campaign lifecycle stays table-first while edits, status changes, and cleanup stay scoped to the selected offer.':
    '活动生命周期始终以表格为主，编辑、状态变更和清理均限定在所选优惠项内。',
  Capability: '能力',
  'Capture the high-level mapping metadata before composing individual translation rules.':
    '在编排具体转换规则之前，先记录高层级映射元数据。',
  'Catalog operations now run on one shared workbench for channels, providers, credentials, and deployable variants.':
    '目录操作现已统一在一个共享工作台内完成，覆盖渠道、提供商、凭据和可部署变体。',
  'Channel details, publications, and pricing remain visible while the directory stays primary.':
    '在目录保持主视图的同时，渠道详情、发布和定价信息持续可见。',
  'Choose how the shell follows light, dark, or system appearance.':
    '选择 shell 如何跟随浅色、深色或系统外观。',
  'Choose the operator workspace language. Dates, numbers, and shared shell copy follow this setting immediately.':
    '选择运营工作区语言。日期、数字和共享 shell 文案会立即跟随该设置切换。',
  'Claw-style low-glare shell with higher contrast.': '低眩光高对比的 Claw 风格 shell。',
  collapsed: '折叠',
  'Collapsed sidebar': '折叠侧边栏',
  'Compact navigation': '紧凑导航',
  'Content region': '内容区域',
  'Continue with': '继续使用以下方式',
  'Control expiration, notes, and whether the plaintext key is generated or provided manually.':
    '控制过期时间、备注，以及明文密钥是自动生成还是手动提供。',
  'control plane settings center': '控制平面设置中心',
  'Create a password': '创建密码',
  'Create a tenant to start assigning projects and issuing gateway keys.':
    '先创建租户，再开始分配项目和签发网关密钥。',
  'Create account': '创建账号',
  'Credential storage and rotation context stay visible alongside the directory.':
    '凭据存储与轮换上下文会与目录一同保持可见。',
  'Current shell posture for the control plane workspace.': '当前控制平面工作区的 shell 姿态。',
  Dark: '深色',
  'Decision id': '决策 ID',
  Decisions: '决策数',
  'Decisions that applied an SLO.': '已应用 SLO 的决策数。',
  'Decisions that degraded the target SLO.': '使目标 SLO 降级的决策数。',
  'Default strategy': '默认策略',
  'Distinct providers selected by the current slice.': '当前范围内被选中的不同提供商数量。',
  'Enter your password': '输入你的密码',
  Entries: '条目数',
  'Every shell preference persists so the control plane reopens with the same workspace and operator posture.':
    '所有 shell 偏好都会持久化保存，因此控制平面会以相同的工作区与运营姿态重新打开。',
  Exhausted: '已耗尽',
  expanded: '展开',
  Expired: '已过期',
  'Expires today.': '今天到期。',
  'Expiring soon': '即将到期',
  'Expiry date has already passed and needs operator review.': '过期日期已过，需要运营人员检查。',
  'Expiry date is not available.': '暂无过期日期。',
  'Follow the device preference automatically.': '自动跟随设备偏好。',
  'Forgot password?': '忘记密码？',
  'Gateway posture': '网关姿态',
  General: '常规',
  GitHub: 'GitHub',
  Global: '全局',
  Google: 'Google',
  'Hidden nav items': '隐藏导航项',
  'Hotspot projects combine request volume, token load, and amount in one ranked view.':
    '热点项目将请求量、令牌负载和金额合并到同一个排序视图中。',
  'Keep per-key route mode, provider pinning, and model mapping aligned with the local overlay behavior.':
    '保持每个密钥的路由模式、提供商固定和模型映射与本地覆盖行为一致。',
  'Keep route posture focused on upstream connectivity and channel exposure. Credentials and model publication remain visible from the main workbench.':
    '让路由姿态聚焦于上游连通性和渠道暴露，同时在主工作台中持续展示凭据与模型发布信息。',
  'Keep the left navigation rail and the right canvas in a single consistent shell contract.':
    '让左侧导航轨和右侧画布保持在统一一致的 shell 契约中。',
  'Keep the left rail expanded or collapse it into icon-only navigation.':
    '保持左侧导航轨展开，或将其折叠为仅图标导航。',
  'Language and locale': '语言与区域设置',
  'Language updates every route label, shell notice, and workspace detail immediately.':
    '语言切换会立即更新每个路由标签、shell 提示和工作区详情。',
  Light: '浅色',
  'live shell summary': '实时 shell 摘要',
  'Local dev credentials are prefilled: {email} / {password}.':
    '本地开发凭据已预填：{email} / {password}。',
  'Mapping policy and rule inventory stay linked to the selected overlay so operators can edit or disable it without leaving the registry.':
    '映射策略与规则清单始终绑定到所选覆盖项，运营人员无需离开注册表即可编辑或停用。',
  'Mint a project-scoped API key in a focused dialog, then reveal the plaintext once for secure handoff.':
    '在聚焦对话框中签发项目级 API 密钥，并仅展示一次明文以便安全交接。',
  Name: '姓名',
  'name@example.com': 'name@example.com',
  Navigation: '导航',
  Neutral: '中性',
  'No account?': '没有账号？',
  'No billing records match the current filters': '没有计费记录符合当前筛选条件',
  'No hotspot projects match the current filters': '没有热点项目符合当前筛选条件',
  'No keys': '无密钥',
  'No model mappings match the current filter': '没有模型映射符合当前筛选条件',
  'No portal users match the current filters': '没有门户用户符合当前筛选条件',
  'No projects': '无项目',
  'No quota policy': '未分配配额策略',
  'No routing decisions match the current filters': '没有路由决策符合当前筛选条件',
  'No settings match your search': '没有设置项符合当前搜索',
  'No tenants available': '暂无可用租户',
  'No usage records match the current filter': '没有用量记录符合当前筛选条件',
  'No usage records match the current filters': '没有用量记录符合当前筛选条件',
  'Not assigned': '未分配',
  'Not recorded': '未记录',
  'Open app to scan': '打开应用扫码',
  'Open the SDKWork app and scan this code to continue without typing credentials.':
    '打开 SDKWork 应用并扫描此二维码，无需输入凭据即可继续。',
  Operator: '运营员',
  'Operator account requests stay inside the control plane. Ask an existing admin to provision {name} access from Users.':
    '运营账号申请会保留在控制平面内处理。请让现有管理员在用户页为 {name} 开通访问权限。',
  'Operator identity, locale, and shell posture summary': '运营身份、语言区域与 shell 姿态摘要',
  'Page {page} of {total}': '第 {page} 页，共 {total} 页',
  'Persistence, content region, and shell continuity': '持久化、内容区域与 shell 连续性',
  'Plaintext is not visible on this device. Create a replacement if you need to copy it again.':
    '当前设备无法查看明文。如需再次复制，请重新创建。',
  'Portal identities ranked by visible request volume, token load, and amount.':
    '按可见请求量、令牌负载和金额对门户身份进行排序。',
  'Portal identities visible in the current slice.': '当前范围内可见的门户身份数。',
  'Portal user': '门户用户',
  'Portal user traffic': '门户用户流量',
  'Project hotspot': '项目热点',
  'Project hotspots': '项目热点',
  'Project-level billing posture with quota status and remaining headroom.':
    '展示项目级计费姿态、配额状态和剩余空间。',
  'Projects currently marked as exhausted.': '当前被标记为耗尽的项目数。',
  'Projects ranked by visible request volume, token load, and amount.':
    '按可见请求量、令牌负载和金额对项目进行排序。',
  'Projects remain the routing, usage, and billing ownership boundary, so edits belong in their own dialog.':
    '项目仍然是路由、用量和计费归属的边界，因此编辑应放在独立对话框中进行。',
  'Projects represented by the current usage slice.': '当前用量范围覆盖的项目数。',
  'Projects represented in the current slice.': '当前范围覆盖的项目数。',
  'Provider bindings, credentials, and adapter posture stay visible while the registry remains primary.':
    '在注册表保持主视图的同时，提供商绑定、凭据和适配器姿态持续可见。',
  'Provider selection, route key, and SLO posture remain visible for every routing decision.':
    '每条路由决策都会持续显示提供商选择、路由键和 SLO 姿态。',
  'Provider selection, route key, and strategy stay visible for routing audits.':
    '在路由审计中，提供商选择、路由键和策略信息始终可见。',
  'Provider-scoped model variants remain easy to inspect and publish into channels.':
    '提供商范围内的模型变体仍然便于检查并发布到渠道。',
  'provider, base url, credential, channel': '提供商、基础地址、凭据、渠道',
  'QR login': '扫码登录',
  'Quick setup keeps the API key workbench aligned with real gateway compatibility endpoints and local client bootstrapping flows.':
    '快速配置让 API 密钥工作台与真实网关兼容端点和本地客户端引导流程保持一致。',
  Quota: '配额',
  'Quota exhausted': '配额已耗尽',
  'Quota healthy': '配额健康',
  'Quota policy': '配额策略',
  Readiness: '就绪度',
  Reason: '原因',
  'Reduce the rail to icon-only navigation without changing the canvas.':
    '将导航轨缩减为仅图标模式，同时不改变画布布局。',
  Region: '区域',
  Remaining: '剩余',
  'Remaining units': '剩余单位',
  'Request-level cost and token accounting for every visible interaction.':
    '针对每个可见交互展示请求级成本与令牌核算。',
  'Request-level metering for the selected project interaction.': '所选项目交互的请求级计量。',
  'Requests: {requests} | Usage units: {units} | Tokens: {tokens}':
    '请求：{requests} | 用量单位：{units} | 令牌：{tokens}',
  'Restoring theme, session, and live control-plane state.': '正在恢复主题、会话和实时控制平面状态。',
  'Review the degraded providers in the table and use the inspector rail to understand channel and credential impact before editing.':
    '请先在表格中审查已降级的提供商，并通过检查轨了解渠道与凭据影响后再编辑。',
  'right canvas': '右侧画布',
  'Route key': '路由键',
  'Route providers keep channel bindings, credentials, and pricing posture attached to the selected registry row.':
    '路由提供商会将渠道绑定、凭据和定价姿态持续绑定在所选注册表行上。',
  'Routing decision log': '路由决策日志',
  'Search and switch settings without leaving the shared desktop shell.':
    '无需离开共享桌面 shell，即可搜索并切换设置。',
  'Selected provider': '已选提供商',
  'Set the initial route mode, optional provider pinning, and optional model mapping.':
    '设置初始路由模式、可选的提供商固定和可选的模型映射。',
  'settings center': '设置中心',
  'Settings center': '设置中心',
  'Shared tables and dialogs replace the old local catalog scaffolding without keeping a second UI system alive.':
    '共享表格和对话框已替换旧的本地目录脚手架，不再保留第二套 UI 系统。',
  Shell: 'Shell',
  'shell continuity': 'shell 连续性',
  'Shell continuity': 'Shell 连续性',
  'shell posture': 'shell 姿态',
  'Show or hide modules while keeping the left navigation rail compact and stable.':
    '在保持左侧导航轨紧凑稳定的同时，显示或隐藏模块。',
  'sidebar and canvas posture': '侧边栏与画布姿态',
  'Sidebar behavior': '侧边栏行为',
  'Sidebar mode': '侧边栏模式',
  'sidebar visibility': '侧边栏可见性',
  'Sidebar visibility, rail behavior, and module exposure': '侧边栏可见性、导航轨行为与模块展示',
  'Sidebar width': '侧边栏宽度',
  'Sign in to manage the control plane': '登录以管理控制平面',
  'single workspace surface': '单一工作区界面',
  'sk-router-live-demo': 'sk-router-live-demo',
  'SLO applied': 'SLO 已应用',
  'SLO degraded': 'SLO 已降级',
  'SLO posture': 'SLO 姿态',
  'SLO stable': 'SLO 稳定',
  Strategy: '策略',
  'Switch between catalog domains and search the active directory.':
    '在目录域之间切换，并搜索当前活动目录。',
  'Synchronizing operator workspace...': '正在同步运营工作区...',
  System: '系统',
  'Tenant creation and editing happen in a dedicated dialog so the registry stays primary on the page.':
    '租户创建与编辑通过专用对话框完成，以保持注册表作为页面主视图。',
  'Tenant operations stay scoped here while project creation and key issuance remain in focused dialogs.':
    '租户操作保持在当前范围内，而项目创建与密钥签发仍放在聚焦对话框中。',
  'The layout stays split into a claw-style left navigation rail and a single right content region, keeping product behavior and visual framing consistent.':
    '布局继续保持为 claw 风格左侧导航轨和单一右侧内容区，确保产品行为与视觉框架一致。',
  'The left rail remains the navigation source of truth and the right canvas remains the only content display region for every admin page.':
    '左侧导航轨始终是唯一导航事实来源，右侧画布始终是所有后台页面唯一的内容展示区域。',
  'The left rail remains the navigation source of truth while labels collapse into icons.':
    '在标签折叠为图标后，左侧导航轨仍然是导航事实来源。',
  'Theme color': '主题色',
  'Theme color updates accent surfaces without changing the shell contract.':
    '主题色会更新强调表面，而不会改变 shell 契约。',
  'Theme mode': '主题模式',
  'Theme mode, accent preset, and shared shell look': '主题模式、强调色预设与共享 shell 外观',
  'Theme posture': '主题姿态',
  'Theme preferences, sidebar width, hidden entries, and collapse state are persisted so the control-plane workspace reopens with the same shell posture.':
    '主题偏好、侧边栏宽度、隐藏项和折叠状态都会被持久化，因此控制平面工作区会以相同 shell 姿态重新打开。',
  'This workspace keeps operator preferences, shell posture, and control plane continuity aligned with claw-studio while preserving router-admin workflows.':
    '当前工作区在保留 router-admin 工作流的同时，使运营偏好、shell 姿态和控制平面连续性与 claw-studio 保持一致。',
  Traffic: '流量',
  'Try a broader query or wider time window to inspect more portal users.':
    '请放宽查询条件或扩大时间窗口，以查看更多门户用户。',
  'Try a broader query or wider time window to surface more hotspot projects.':
    '请放宽查询条件或扩大时间窗口，以呈现更多热点项目。',
  'Try a broader query to inspect more billing records.': '请放宽查询条件，以查看更多计费记录。',
  'Try a different keyword or browse the navigation without a search term.':
    '请尝试其他关键词，或在不输入搜索词的情况下直接浏览导航。',
  'Try a different search query or broaden the mapping status filter.':
    '请尝试其他搜索条件，或放宽映射状态筛选。',
  Unassigned: '未分配',
  'Unassigned project': '未分配项目',
  'Unassigned tenant': '未分配租户',
  Unlimited: '无限制',
  'Usage attribution stays attached to the mapped workspace and identity.':
    '用量归属始终绑定到已映射的工作区和身份。',
  'Usage inspection keeps the active key filter and time window attached to the currently selected ledger row.':
    '用量检查会将当前激活的密钥筛选和时间窗口持续绑定到所选台账行。',
  'Usage ledger': '用量台账',
  'Usage request': '用量请求',
  'User traffic leaderboard': '用户流量排行',
  'Users that remain active.': '仍处于活跃状态的用户数。',
  'Visible hotspots': '可见热点',
  'Visible requests in the current slice.': '当前范围内可见的请求数。',
  'Visible routes': '可见路由',
  'Visible routing decisions.': '可见路由决策数。',
  'Visible users': '可见用户',
  'Workspace owner': '工作区负责人',
  'workspace persistence': '工作区持久化',
  your: '你的',
};

const ADMIN_ZH_DYNAMIC_METADATA_TRANSLATIONS: Record<string, string> = {
  'Create access': '创建访问',
  'Create operator access': '创建运营访问',
  'Password reset links are not enabled for this workspace yet. Continue back to sign in with your operator email.':
    '当前工作区尚未启用密码重置链接。请返回并使用运营邮箱登录。',
  'Recover access': '恢复访问',
  Recovery: '恢复',
  'Request access': '申请访问',
  'Request operator access and continue into the router control plane once an existing admin provisions your identity.':
    '申请运营访问，待现有管理员为你的身份开通权限后继续进入路由控制平面。',
  'Sign in to continue to your router admin workspace.': '登录以继续进入你的路由管理工作区。',
  'Sign up': '注册',
  'Welcome back': '欢迎回来',
  Auth: '鉴权',
  'Claude Code': 'Claude Code',
  'Claude Code uses the Anthropic-compatible route exposed by the gateway and keeps the same Api key.':
    'Claude Code 使用网关暴露的 Anthropic 兼容路由，并继续沿用同一个 API 密钥。',
  Codex: 'Codex',
  Config: '配置',
  Gemini: 'Gemini',
  'Gemini CLI uses the gateway Gemini-compatible compatibility routes while keeping this Api key as the only secret.':
    'Gemini CLI 使用网关的 Gemini 兼容路由，并将当前 API 密钥作为唯一密钥。',
  OpenClaw: 'OpenClaw',
  'OpenClaw writes a provider manifest into the selected local instances and points them at the routed gateway endpoint.':
    'OpenClaw 会将提供商清单写入所选本地实例，并让实例指向当前路由后的网关端点。',
  OpenCode: 'OpenCode',
  'OpenCode uses the OpenAI-compatible provider block and the same routed Api key.':
    'OpenCode 使用 OpenAI 兼容的提供商配置块，并沿用同一个路由后的 API 密钥。',
  'Provider manifest': '提供商清单',
  'Audio second': '音频秒',
  'Image generated': '图片生成',
  'Million tokens': '百万令牌',
  'Music track': '音乐轨道',
  Request: '请求',
  'Thousand tokens': '千令牌',
  'Video minute': '视频分钟',
  'Api Key': 'API 密钥',
  'Api key registry with key-level route posture and usage visibility':
    '带有密钥级路由姿态和用量可见性的 API 密钥注册表。',
  'API Router': 'API 路由',
  Audit: '审计',
  'Campaign and discount code operations': '活动与优惠码运营',
  'Channel and upstream route registry in the claw apirouter model':
    'claw apirouter 模型下的渠道与上游路由注册表。',
  'Channels, providers, and model exposure': '渠道、提供商与模型暴露',
  Control: '控制',
  'Control Plane': '控制平面',
  Coupons: '优惠券',
  'Global health, alerts, and operator shortcuts': '全局健康、告警和运营快捷入口',
  Growth: '增长',
  Guardrail: '护栏',
  'Health snapshots, reloads, and runtime posture': '健康快照、重载与运行时姿态',
  Identity: '身份',
  Mesh: '网格',
  'Model Mapping': '模型映射',
  Operations: '运维',
  'Operator and portal user management': '运营员与门户用户管理',
  'Overlay model mapping rules for gateway clients and OpenClaw':
    '面向网关客户端和 OpenClaw 的覆盖模型映射规则。',
  Preferences: '偏好',
  'Project, key, route, and model request-frequency policies':
    '项目、密钥、路由和模型的请求频率策略。',
  'Rate Limits': '限流',
  'Request history, token volume, and CSV export by Api key':
    '按 API 密钥查看请求历史、令牌量和 CSV 导出。',
  Route: '路由',
  'Route Config': '路由配置',
  'Routing Mesh': '路由网格',
  Tenants: '租户',
  'Tenants, projects, and gateway keys': '租户、项目与网关密钥',
  'Theme mode, theme color, and sidebar preferences': '主题模式、主题颜色与侧边栏偏好',
  'Usage Records': '用量记录',
  'Usage, billing, and request-log visibility': '用量、计费与请求日志可见性',
  Users: '用户',
  'Workspace Ops': '工作区运营',
  'Active models': '活跃模型',
  'Admin API base': '管理 API 基址',
  'Combined operator and portal inventory.': '运营员与门户用户合并总量。',
  'Coupon campaigns are live-backed': '优惠活动已接入实时后端',
  'Coupon operations now persist through the admin control plane instead of local workspace state.':
    '优惠券操作现已通过管理控制平面持久化，而不再依赖本地工作区状态。',
  'Credential coverage': '凭据覆盖率',
  'Independent admin project talking to the operator control plane.':
    '独立管理项目正在与运营控制平面对接。',
  'Managed users': '受管用户',
  'Models currently exposed through the routing catalog.': '当前通过路由目录暴露的模型数。',
  'No model catalog entries': '暂无模型目录条目',
  'One or more managed runtimes are unhealthy. Review the Operations module.':
    '一个或多个受管运行时不健康。请检查 Operations 模块。',
  'Projects with exhausted quota': '配额耗尽的项目',
  'Provider credentials are missing': '缺少提供商凭据',
  'Providers currently backed by at least one upstream credential.':
    '当前已由至少一个上游凭据支撑的提供商数量。',
  'Request volume': '请求量',
  'Runtime health degradation detected': '检测到运行时健康降级',
  'The routing layer has no published models. Create or upsert models in Catalog.':
    '路由层尚未发布任何模型。请在 Catalog 中创建或更新模型。',
  'Total requests recorded by the usage summary.': '用量汇总记录的总请求数。',
  'green-tech': '绿色科技',
  lobster: '龙虾红',
  rose: '玫瑰',
  'tech-blue': '科技蓝',
  violet: '紫罗兰',
  zinc: '锌灰',
  Billing: '计费',
  Routing: '路由',
};

const ADMIN_ZH_CATALOG_TABLE_TRANSLATIONS: Record<string, string> = {
  '{count} channels': '{count} 个渠道',
  '{count} publications': '{count} 个发布项',
  '{tenant} / {provider}': '{tenant} / {provider}',
  'Channel operations stay table-first while publication and pricing work remains attached to the selected drawer context.':
    '渠道操作保持表格优先，发布与定价工作继续绑定在当前选中的抽屉上下文中。',
  'Channel publications, provider coverage, and pricing stay attached to the selected directory record.':
    '渠道发布、提供商覆盖和定价信息保持绑定在当前选中的目录记录上。',
  'Credential lifecycle remains scoped to the selected provider reference without leaving the directory workflow.':
    '凭证生命周期保持限定在当前选中的提供商引用中，无需离开目录工作流。',
  'Delete {label}. This action removes the selected catalog record.':
    '删除 {label}。此操作会移除当前选中的目录记录。',
  'New channel': '新建渠道',
  'New credential': '新建凭证',
  'Non-streaming': '非流式',
  'Provider posture stays connected to channel bindings, adapter configuration, and credential rotation.':
    '提供商状态持续关联渠道绑定、适配器配置和凭证轮换。',
  'Provider variants can be inspected and published into the active channel without leaving the directory table.':
    '提供商变体可以在不离开目录表格的情况下查看并发布到当前渠道。',
  'Provider-scoped variants available for publication.': '可供发布的提供商作用域变体。',
  'Provider-specific pricing rows stay aligned with the selected publication.':
    '提供商定价行与当前选中的发布项保持对齐。',
  'Publish variant': '发布变体',
  Variant: '变体',
};

const ADMIN_TRANSLATIONS: Record<AdminLocale, Record<string, string>> = {
  'en-US': {},
  'zh-CN': {
    ...ADMIN_ZH_EXPANDED_TRANSLATIONS,
    ...ADMIN_ZH_COMPLETION_TRANSLATIONS,
    ...ADMIN_ZH_DYNAMIC_METADATA_TRANSLATIONS,
    ...ADMIN_ZH_CATALOG_TABLE_TRANSLATIONS,
    English: '英语',
    'Simplified Chinese': '简体中文',
    Language: '语言',
    Workspace: '工作区',
    Search: '搜索',
    Refresh: '刷新',
    Settings: '设置',
    Confirm: '确认',
    Cancel: '取消',
    Guest: '访客',
    'Sign in': '登录',
    'Sign out': '退出登录',
    'Profile settings': '个人设置',
    'Signed in': '已登录',
    'Collapse sidebar': '折叠侧边栏',
    'Expand sidebar': '展开侧边栏',
    'Open user menu': '打开用户菜单',
    'Close user menu': '关闭用户菜单',
    'Control plane operator': '控制平面运营员',
    'Router Admin': '路由管理后台',
    'Control plane': '控制平面',
    Overview: '概览',
    'Live control-plane posture': '实时控制平面态势',
    'Global health, operator alerts, and the most active workspace surfaces from the live control plane.':
      '从实时控制平面查看全局健康、运营告警和最活跃的工作区。',
    'Metrics and alerts refresh from the same shared shell contract used across the rest of the admin workspace.':
      '指标与告警会通过与管理后台其他区域一致的共享壳层协议刷新。',
    'Operator alerts': '运营告警',
    'Alerts are generated from live billing, runtime, catalog, and workspace health signals.':
      '告警由实时计费、运行时、目录以及工作区健康信号生成。',
    'Traffic leaders': '流量领先项',
    'The right rail keeps the busiest users and projects visible without leaving the main overview.':
      '右侧栏会持续展示最繁忙的用户和项目，无需离开主概览。',
    'Top portal users': '门户活跃用户',
    'Hottest projects': '最热项目',
    'Search users': '搜索用户',
    'Search tenants': '搜索租户',
    'Search campaigns': '搜索活动',
    'Search API keys': '搜索 API 密钥',
    'Search providers': '搜索供应商',
    'Search mappings': '搜索映射',
    'Search usage': '搜索用量',
    'Search traffic': '搜索流量',
    'Search operations': '搜索运维',
    'Search catalog': '搜索目录',
    'Search settings': '搜索设置',
    'Recent window': '最近时间窗口',
    'Traffic view': '流量视图',
    'Operational view': '运维视图',
    'Reset filters': '重置筛选',
    'Reset search': '重置搜索',
    'Clear search': '清空搜索',
    'Clear filters': '清空筛选',
    'Export CSV': '导出 CSV',
    'Export usage CSV': '导出用量 CSV',
    'Reload all runtimes': '重载全部运行时',
    'Targeted reload': '定向重载',
    'Run targeted reload': '执行定向重载',
    User: '用户',
    Type: '类型',
    Status: '状态',
    Requests: '请求数',
    Tokens: '令牌数',
    Units: '单位数',
    Amount: '金额',
    'Amount: {amount}': '金额：{amount}',
    'Requests: {count}': '请求：{count}',
    'Tokens: {count}': '令牌：{count}',
    'Units: {count}': '单位：{count}',
    'User type': '用户类型',
    'All users': '全部用户',
    Operators: '运营员',
    'Portal users': '门户用户',
    'All statuses': '全部状态',
    Active: '启用',
    Disabled: '停用',
    'name, email, tenant, project': '姓名、邮箱、租户、项目',
    'New Operator': '新建运营员',
    'New Portal User': '新建门户用户',
    'Delete now': '立即删除',
    'Delete operator account': '删除运营员账号',
    'Delete portal account': '删除门户账号',
    'Remove {name} ({email}) from the directory. This action cannot be undone from this console.':
      '将 {name}（{email}）从目录中移除。此操作无法在当前控制台撤销。',
    'control-plane': '控制平面',
    'shared operator context': '共享运营上下文',
    '{count} visible': '{count} 条可见',
    '{count} active': '{count} 个启用',
    '{count} users': '{count} 个用户',
    '{count} operators': '{count} 个运营员',
    '{count} portal users': '{count} 个门户用户',
    'Showing {start} - {end} of {total}': '显示 {start} - {end} / 共 {total}',
    'No users match the current filter': '没有用户符合当前筛选条件',
    'Try a different keyword or broaden the user type and status filters.':
      '请尝试其他关键词，或放宽用户类型与状态筛选。',
    Edit: '编辑',
    Delete: '删除',
    Enable: '启用',
    Disable: '停用',
    Restore: '恢复',
    'Operator identities manage the control plane directly.': '运营员身份可直接管理控制平面。',
    'Portal identities inherit their tenant and project scope.': '门户身份会继承所属租户与项目范围。',
    'Usage units': '用量单位',
    'User information': '用户信息',
    'Basic identity and access information for the selected user.': '所选用户的基础身份与访问控制信息。',
    'Display Name': '显示名称',
    'Display name': '显示名称',
    Email: '邮箱',
    Role: '角色',
    Tenant: '租户',
    Project: '项目',
    'Workspace attribution and scope for the selected user.': '所选用户的工作区归属与范围。',
    'Project attribution': '项目归属',
    'Live usage and billing posture for the selected portal user.': '所选门户用户的实时用量与计费状态。',
    'Project name': '项目名称',
    'Project requests': '项目请求数',
    'Used units': '已用单位',
    'Protected identities': '受保护身份',
    'Bootstrap operators and the current active session should not be removed from the directory.':
      '引导运营员以及当前活跃会话不应从目录中移除。',
    'Edit operator': '编辑运营员',
    'Create operator': '创建运营员',
    'Operators manage catalog, traffic, and runtime posture. Keep this population tightly controlled.':
      '运营员负责目录、流量和运行时态势管理，应严格控制该类账号数量。',
    'Operator accounts stay minimal, high-trust, and easy to audit.':
      '运营员账号应保持精简、高信任且易于审计。',
    'Operator profile': '运营员资料',
    'New password': '新密码',
    Password: '密码',
    'Leave blank to preserve the current password.': '留空以保留当前密码。',
    'Set a strong operator password.': '请设置高强度运营员密码。',
    'Save operator': '保存运营员',
    'Edit portal user': '编辑门户用户',
    'Create portal user': '创建门户用户',
    'Portal identities are scoped to a tenant and project so usage, billing, and request posture remain attributable.':
      '门户身份绑定到租户和项目，以便用量、计费与请求状态都可归因。',
    'Capture the user identity first, then bind it to a specific workspace.':
      '先录入用户身份，再绑定到具体工作区。',
    'Portal identity': '门户身份',
    'Leave blank to keep the current secret.': '留空以保留当前密码。',
    'Set an initial portal password.': '设置门户初始密码。',
    'Tenant and project scope determine where traffic and billing attribution land.':
      '租户和项目范围决定流量与计费归属落点。',
    'Workspace binding': '工作区绑定',
    'Workspace tenant': '工作区租户',
    'Workspace project': '工作区项目',
    'Selected workspace posture': '所选工作区态势',
    'Unassigned workspace': '未分配工作区',
    '{workspace} | Requests: {requests} | Usage units: {units} | Tokens: {tokens}':
      '{workspace} | 请求：{requests} | 用量单位：{units} | 令牌：{tokens}',
    'Save portal user': '保存门户用户',
    'New tenant': '新建租户',
    'New project': '新建项目',
    'Issue gateway key': '签发网关密钥',
    'tenant, project, environment, key label': '租户、项目、环境、密钥标签',
    '{count} projects': '{count} 个项目',
    '{count} active keys': '{count} 个启用密钥',
    'Delete workspace resource': '删除工作区资源',
    'Delete {label}. This permanently removes the selected resource from the workspace registry.':
      '删除 {label}。这会将所选资源从工作区注册表中永久移除。',
    'Tenant profile': '租户资料',
    'Tenant identifiers should remain stable because projects, portal users, and keys inherit this boundary.':
      '租户标识应保持稳定，因为项目、门户用户和密钥都继承该边界。',
    'Tenant id': '租户 ID',
    'Tenant name': '租户名称',
    'Save tenant': '保存租户',
    'Create tenant': '创建租户',
    'Edit tenant': '编辑租户',
    'Project profile': '项目资料',
    'Projects inherit tenant scope and become the primary boundary for usage, billing, and key issuance.':
      '项目继承租户范围，并成为用量、计费和密钥签发的主要边界。',
    'Project id': '项目 ID',
    'Selected project posture': '所选项目态势',
    'Save project': '保存项目',
    'Create project': '创建项目',
    'Edit project': '编辑项目',
    'Scope determines which tenant and project own the new key.': '范围决定新密钥归属的租户和项目。',
    'Key scope': '密钥范围',
    Environment: '环境',
    Production: '生产',
    Staging: '预发',
    Development: '开发',
    'Inventory metadata': '库存元数据',
    'Labels and notes keep the key inventory readable for operators.':
      '标签和备注可帮助运营员更清晰地管理密钥库存。',
    'Key label': '密钥标签',
    'Production App Key': '生产应用密钥',
    'Expires at (ms)': '过期时间（毫秒）',
    Notes: '备注',
    'Retained for admin inventory': '保留用于后台库存管理',
    'Issue key': '签发密钥',
    'Plaintext key ready': '明文密钥已生成',
    'Store this secret now. The control plane persists both the hashed and raw key.':
      '请立即保存该密钥。控制平面会保存哈希值和原始密钥。',
    'Issued scope': '签发范围',
    'Plaintext key': '明文密钥',
    'Copy the key now. It will not be revealed again in this dialog.':
      '请立即复制该密钥。关闭后此对话框将不再显示。',
    'Copy key': '复制密钥',
    Close: '关闭',
    Projects: '项目',
    'Active keys': '启用密钥',
    'Workspace posture': '工作区态势',
    'Active projects, portal users, and gateway coverage for the selected tenant.':
      '所选租户的活跃项目、门户用户与网关覆盖情况。',
    'Project footprint': '项目覆盖',
    'Gateway coverage': '网关覆盖',
    'Traffic footprint': '流量分布',
    '{count} projects attached': '已关联 {count} 个项目',
    '{active} active / {total} total': '{active} 个启用 / 共 {total} 个',
    '{count} requests': '{count} 个请求',
    '{count} tokens': '{count} 个令牌',
    'At least one project exists, so gateway key issuance can proceed immediately.':
      '至少已存在一个项目，因此可以立即签发网关密钥。',
    'Issue a gateway key only after at least one project exists for the selected tenant.':
      '仅当所选租户至少存在一个项目后，才可签发网关密钥。',
    'Key issuance ready': '可签发密钥',
    'Key issuance guardrail': '密钥签发约束',
    Campaign: '活动',
    Offer: '优惠',
    'Remaining quota': '剩余额度',
    'Quota health': '额度健康度',
    Expiry: '过期',
    Archived: '已归档',
    'Campaign state': '活动状态',
    'All campaigns': '全部活动',
    'At risk': '风险中',
    'code, audience, note': '兑换码、受众、备注',
    '{count} live': '{count} 个在线',
    'New coupon': '新建优惠券',
    'Delete coupon': '删除优惠券',
    'Delete coupon campaign': '删除优惠券活动',
    'Remove {code} from the campaign roster. This permanently deletes the offer from the admin control plane.':
      '将 {code} 从活动列表中移除。这会从管理控制平面永久删除该优惠。',
    'Create coupon': '创建优惠券',
    'Edit coupon campaign': '编辑优惠券活动',
    'Use one modal for both launch and revision so the roster always stays primary in the workspace.':
      '创建和编辑共用一个弹窗，保持列表始终作为工作区主视图。',
    'Campaign profile': '活动资料',
    'Campaign identifiers, offer copy, and audience targeting live together for easier operator review.':
      '活动标识、优惠文案和受众范围统一展示，便于运营审核。',
    'Stored in uppercase for consistency across support and redemption flows.':
      '统一以大写保存，便于支持和兑换流程保持一致。',
    'Coupon code': '优惠码',
    'Discount label': '优惠标签',
    Audience: '受众',
    'Expires on': '到期日期',
    'Operator note': '运营备注',
    'Operator notes capture campaign intent, guardrails, and support context.':
      '运营备注用于记录活动意图、约束和支持上下文。',
    'Save coupon': '保存优惠券',
    'Campaign posture': '活动态势',
    Discount: '优惠',
    'Expiry window': '到期窗口',
    'Support and campaign operators can use this window to stage renewals or retire the offer.':
      '支持和活动运营可根据该窗口安排续期或下线优惠。',
    'No coupons match the current filter': '没有优惠券符合当前筛选条件',
    'Try a different campaign state or broaden the search query.':
      '请尝试其他活动状态，或放宽搜索条件。',
    '{count} campaigns': '{count} 个活动',
    '{count} at risk': '{count} 个风险项',
    '{count} audiences': '{count} 类受众',
    '{count} quota': '{count} 配额',
    '{code} expires next': '{code} 即将最先到期',
    'Showing {start} - {end} of {total} | {archived} archived | {expiringSoon} expiring soon':
      '显示 {start} - {end} / 共 {total} | 已归档 {archived} | 即将到期 {expiringSoon}',
    'Create API key': '创建 API 密钥',
    'label, workspace, hashed key, provider': '标签、工作区、哈希密钥、供应商',
    '{count} custom routes': '{count} 条自定义路由',
    'Refresh workspace': '刷新工作区',
    'Delete key': '删除密钥',
    'Delete API key': '删除 API 密钥',
    Provider: '供应商',
    Channels: '渠道',
    Coverage: '覆盖',
    Credentials: '凭据',
    Health: '健康',
    'Base URL': '基础地址',
    'No credentials': '无凭据',
    'All channels': '全部渠道',
    'All providers': '全部供应商',
    'Healthy only': '仅健康',
    'Degraded only': '仅降级',
    '{count} degraded': '{count} 个降级',
    '{count} total': '共 {count} 个',
    'New route provider': '新建路由供应商',
    'Delete provider': '删除供应商',
    'Delete route provider': '删除路由供应商',
    'Delete {name}. Review route bindings, pricing, and downstream channel coverage before removing the provider.':
      '删除 {name}。移除前请检查路由绑定、定价以及下游渠道覆盖情况。',
    Mapping: '映射',
    'No description': '无描述',
    'Effective window': '生效窗口',
    Rules: '规则',
    'name, model, channel': '名称、模型、渠道',
    'All mappings': '全部映射',
    'New model mapping': '新建模型映射',
    'Refresh overlay': '刷新覆盖层',
    '{count} rules': '{count} 条规则',
    'Delete mapping': '删除映射',
    'Delete model mapping': '删除模型映射',
    'Delete {name}. Any API key overlay using this mapping will be detached automatically.':
      '删除 {name}。所有使用该映射的 API 密钥覆盖配置都会自动解绑。',
    Model: '模型',
    'Input tokens': '输入令牌',
    'Output tokens': '输出令牌',
    'Total tokens': '总令牌',
    Created: '创建时间',
    'project, model, provider': '项目、模型、供应商',
    'All API keys': '全部 API 密钥',
    'Time range': '时间范围',
    'All time': '全部时间',
    'Last 24 hours': '最近 24 小时',
    'Last 7 days': '最近 7 天',
    'Last 30 days': '最近 30 天',
    Catalog: '目录',
    'Catalog filters': '目录筛选',
    'Catalog directory': '目录清单',
    'Catalog area': '目录区域',
    'name, id, provider, credential': '名称、ID、供应商、凭据',
    'Delete catalog item': '删除目录项',
    'project, provider, model, route, user': '项目、供应商、模型、路由、用户',
    'provider, runtime, instance, message': '供应商、运行时、实例、消息',
    Runtimes: '运行时',
    to: '至',
    'Try a different keyword or broaden the project and state filters.': '尝试更换关键词，或放宽项目与状态筛选条件。',
    'No rate limit policies match the current filter': '当前筛选条件下没有匹配的限流策略',
    Inspect: '查看',
    '{count} policies': '{count} 个策略',
    '{count} enabled': '{count} 个已启用',
    '{count} live windows': '{count} 个实时窗口',
    '{count} exceeded': '{count} 个已超限',
    'Project-wide': '项目级',
    Exceeded: '已超限',
    Idle: '空闲',
    Limit: '限制',
    '{count}s window': '{count} 秒窗口',
    Burst: '突发',
    'Requests allowed before hard limiting.': '进入硬限制前允许的突发请求数。',
    Remaining: '剩余',
    '{count} requests observed': '已观测到 {count} 次请求',
    'Waiting for the first live window': '等待首个实时窗口',
    'Applied scope': '生效范围',
    'The router evaluates project scope first, then optional key, route, and model qualifiers.':
      '路由器会先按项目范围评估，再叠加可选的密钥、路由和模型限定条件。',
    'Policy ID': '策略 ID',
    Route: '路由',
    'Any route': '任意路由',
    'Any model': '任意模型',
    Updated: '更新时间',
    'Live window': '实时窗口',
    'Live counters are refreshed from the gateway snapshot already loaded into the admin workspace.':
      '实时计数来自当前已加载到管理工作台的网关快照。',
    'Window start': '窗口开始',
    'Window end': '窗口结束',
    'Observed update': '最近观测更新时间',
    State: '状态',
    'Within threshold': '未超阈值',
    'This policy has not emitted a live window yet. It will appear here after the first matching request reaches the router.':
      '该策略尚未生成实时窗口。首个匹配请求到达路由器后会显示在这里。',
    'Waiting for traffic': '等待流量进入',
    'The router applies the strongest matching policy. Use project-wide rules for a safe default and narrower key, route, or model scopes for exceptions.':
      '路由器会应用匹配度最高的策略。建议使用项目级规则作为安全默认值，再用更细粒度的密钥、路由或模型范围处理例外情况。',
    'Create policy': '创建策略',
    'Define the rate envelope once and optionally narrow it by API key, route, or model.':
      '一次定义限流边界，并可按 API 密钥、路由或模型进一步收窄范围。',
    'Choose the project boundary, then optionally add a narrower key, route, or model scope.':
      '先选择项目边界，再按需补充更细粒度的密钥、路由或模型范围。',
    'Leave empty to cover every route inside the selected project scope.':
      '留空则覆盖所选项目范围内的所有路由。',
    'Leave empty to cover every model routed through this scope.':
      '留空则覆盖该范围内经过路由的所有模型。',
    'Policy envelope': '策略边界',
    'Set the identifier, sustained window, and burst allowance that the gateway should enforce.':
      '设置网关需要执行的策略标识、持续窗口和突发额度。',
    'Requests per window': '每窗口请求数',
    'Window seconds': '窗口秒数',
    'Burst requests': '突发请求数',
    'Disable this if you want to register the policy now but keep enforcement inactive.':
      '如果你希望现在先登记策略但暂不生效，请关闭此项。',
    'Capture operator intent so future reviews can explain why this policy exists.':
      '记录操作意图，方便后续审查解释该策略为何存在。',
    Policy: '策略',
    Scope: '范围',
    '{count} req / {seconds}s': '{count} 次请求 / {seconds} 秒',
    'Burst {count}': '突发 {count}',
    '{used} used | {remaining} left': '已使用 {used} | 剩余 {remaining}',
    'Configured rate envelopes across the gateway surface.': '当前网关表面已配置的限流边界数量。',
    'Policies currently participating in live enforcement.': '当前参与实时限流生效的策略数量。',
    'Policies that already have observed traffic in the current snapshot.':
      '当前快照中已经观测到流量的策略数量。',
    'Live windows that have crossed the configured request ceiling.':
      '实时窗口内已超过配置请求上限的策略数量。',
    'Search rate limits': '搜索限流策略',
    'policy id, project, route, model, key': '策略 ID、项目、路由、模型、密钥',
    'All projects': '全部项目',
    'All states': '全部状态',
  },
};

function interpolate(text: string, values?: TranslationValues) {
  if (!values) {
    return text;
  }

  return Object.entries(values).reduce(
    (result, [key, value]) => result.replaceAll(`{${key}}`, String(value)),
    text,
  );
}

function resolveTranslation(locale: AdminLocale, text: string) {
  return ADMIN_TRANSLATIONS[locale][text] ?? text;
}

function normalizeLocale(value: string | null | undefined): AdminLocale {
  if (!value) {
    return 'en-US';
  }

  return value.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US';
}

function resolveInitialLocale(): AdminLocale {
  if (typeof window === 'undefined') {
    return activeAdminLocale;
  }

  try {
    const persisted = window.localStorage.getItem(ADMIN_I18N_STORAGE_KEY);
    if (persisted) {
      return normalizeLocale(persisted);
    }
  } catch {
    // Ignore storage access failures and fall back to browser locale.
  }

  return normalizeLocale(window.navigator.language);
}

export function translateAdminText(text: string, values?: TranslationValues) {
  return interpolate(resolveTranslation(activeAdminLocale, text), values);
}

export function formatAdminDateTime(value?: number | null) {
  if (!value) {
    return '-';
  }

  return new Intl.DateTimeFormat(activeAdminLocale, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value));
}

export function formatAdminNumber(value: number) {
  return new Intl.NumberFormat(activeAdminLocale).format(value);
}

export function formatAdminCurrency(value: number, fractionDigits = 2) {
  return new Intl.NumberFormat(activeAdminLocale, {
    currency: 'USD',
    maximumFractionDigits: fractionDigits,
    minimumFractionDigits: fractionDigits,
    style: 'currency',
  }).format(value);
}

export function AdminI18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocale] = useState<AdminLocale>(resolveInitialLocale);

  useEffect(() => {
    activeAdminLocale = locale;

    if (typeof document !== 'undefined') {
      document.documentElement.lang = locale;
    }

    if (typeof window !== 'undefined') {
      try {
        window.localStorage.setItem(ADMIN_I18N_STORAGE_KEY, locale);
      } catch {
        // Ignore storage write failures.
      }
    }
  }, [locale]);

  const value = useMemo<AdminI18nContextValue>(
    () => ({
      formatCurrency: (value, fractionDigits) =>
        new Intl.NumberFormat(locale, {
          currency: 'USD',
          maximumFractionDigits: fractionDigits ?? 2,
          minimumFractionDigits: fractionDigits ?? 2,
          style: 'currency',
        }).format(value),
      formatDateTime: (value) => {
        if (!value) {
          return '-';
        }

        return new Intl.DateTimeFormat(locale, {
          dateStyle: 'medium',
          timeStyle: 'short',
        }).format(new Date(value));
      },
      formatNumber: (value) => new Intl.NumberFormat(locale).format(value),
      locale,
      setLocale,
      t: (text, values) => interpolate(resolveTranslation(locale, text), values),
    }),
    [locale],
  );

  return <AdminI18nContext.Provider value={value}>{children}</AdminI18nContext.Provider>;
}

export function useAdminI18n(): AdminI18nContextValue {
  const context = useContext(AdminI18nContext);

  if (!context) {
    throw new Error('Admin i18n hooks must be used inside AdminI18nProvider.');
  }

  return context;
}
