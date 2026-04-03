export interface PortalModelCatalogItem {
  id: string;
  name: string;
  provider: string;
  modalities: string[];
  capability: string;
  contextWindow: string;
  price: string;
  latencyClass: string;
  summary: string;
}

export const portalModelCatalog: PortalModelCatalogItem[] = [
  {
    id: 'doubao-seed-1-6',
    name: 'Doubao Seed 1.6',
    provider: 'Volcengine',
    modalities: ['Text', 'Vision'],
    capability: 'General reasoning and multimodal content understanding',
    contextWindow: '256K',
    price: '$0.35 / 1M tokens',
    latencyClass: 'Low',
    summary: 'A production-friendly Volcengine model for chat, tool workflows, and image-aware prompting.',
  },
  {
    id: 'qwen-max',
    name: 'Qwen Max',
    provider: 'Alibaba Cloud',
    modalities: ['Text', 'Vision', 'Audio'],
    capability: 'Multimodal instruction following and enterprise assistant workflows',
    contextWindow: '1M',
    price: '$0.80 / 1M tokens',
    latencyClass: 'Medium',
    summary: 'Alibaba Cloud flagship coverage for multilingual chat, image parsing, and voice-assisted flows.',
  },
  {
    id: 'ernie-4-5',
    name: 'ERNIE 4.5 Turbo',
    provider: 'Baidu AI Cloud',
    modalities: ['Text', 'Vision'],
    capability: 'Search-grounded reasoning and Chinese knowledge tasks',
    contextWindow: '128K',
    price: '$0.42 / 1M tokens',
    latencyClass: 'Low',
    summary: 'Baidu AI Cloud option tuned for Chinese retrieval, enterprise knowledge work, and document analysis.',
  },
  {
    id: 'deepseek-r1',
    name: 'DeepSeek R1',
    provider: 'DeepSeek',
    modalities: ['Text'],
    capability: 'Long-form reasoning, code generation, and chain-of-thought heavy tasks',
    contextWindow: '128K',
    price: '$0.55 / 1M tokens',
    latencyClass: 'Medium',
    summary: 'Reasoning-first model appropriate for code assistants, analyst flows, and decision support.',
  },
  {
    id: 'claude-3-7-sonnet',
    name: 'Claude 3.7 Sonnet',
    provider: 'Anthropic',
    modalities: ['Text', 'Vision'],
    capability: 'Agentic orchestration, coding, and document understanding',
    contextWindow: '200K',
    price: '$3.00 / 1M tokens',
    latencyClass: 'Medium',
    summary: 'Strong for workspace automation, coding copilots, and long-context business tasks.',
  },
  {
    id: 'gemini-2-5-pro',
    name: 'Gemini 2.5 Pro',
    provider: 'Google',
    modalities: ['Text', 'Vision', 'Audio'],
    capability: 'Massive context, multimodal planning, and advanced tool use',
    contextWindow: '1M',
    price: '$2.50 / 1M tokens',
    latencyClass: 'Medium',
    summary: 'Broad multimodal coverage for assistants that need text, screenshots, audio, and large context.',
  },
  {
    id: 'text-embedding-3-large',
    name: 'text-embedding-3-large',
    provider: 'OpenAI',
    modalities: ['Embedding'],
    capability: 'High-quality semantic retrieval and vector search indexing',
    contextWindow: 'N/A',
    price: '$0.13 / 1M tokens',
    latencyClass: 'Low',
    summary: 'Embedding model for retrieval pipelines, semantic ranking, and knowledge grounding.',
  },
  {
    id: 'whisper-large-v3',
    name: 'Whisper Large V3',
    provider: 'OpenAI',
    modalities: ['Audio'],
    capability: 'Speech recognition, transcript pipelines, and meeting ingestion',
    contextWindow: 'N/A',
    price: '$0.006 / min',
    latencyClass: 'Low',
    summary: 'Audio-first model for transcription pipelines, call summaries, and voice workflow ingestion.',
  },
];
