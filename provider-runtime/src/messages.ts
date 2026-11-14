import type { AssistantMessage, Context, Message, Model, Tool } from '@oh-my-pi/pi-ai';

type Block =
  | { type: 'text'; text: string }
  | { type: 'thinking'; thinking: string; signature: string }
  | { type: 'redacted_thinking'; data: string }
  | { type: 'tool_use'; id: string; name: string; input: Record<string, unknown> }
  | { type: 'tool_extra'; tool_use_id: string; extra: Record<string, unknown> }
  | { type: 'tool_result'; tool_use_id: string; content: string; is_error: boolean }
  | { type: 'provider_data'; provider: string; data: Record<string, unknown> };
type WireMessage = { role: 'user' | 'assistant'; blocks: Block[] };
export type WireRequest = { system: string; messages: WireMessage[]; tools: { name: string; description: string; schema: Tool['parameters'] }[]; effort: string; max_tokens: number };

export function toContext(request: WireRequest, model: Model): Context {
  const messages: Message[] = [];
  const toolNames = new Map<string, string>();
  for (const msg of request.messages) {
    if (msg.role === 'assistant') {
      const extras = new Map(msg.blocks.flatMap(b => b.type === 'tool_extra' ? [[b.tool_use_id, b.extra] as const] : []));
      const content: AssistantMessage['content'] = msg.blocks.flatMap((block): AssistantMessage['content'] => {
        switch (block.type) {
          case 'text': return [{ type: 'text', text: block.text }];
          case 'thinking': return [{ type: 'thinking', thinking: block.thinking, thinkingSignature: block.signature || undefined }];
          case 'redacted_thinking': return [{ type: 'redactedThinking', data: block.data }];
          case 'tool_extra': case 'provider_data': return [];
          case 'tool_use': {
            toolNames.set(block.id, block.name);
            const extra = extras.get(block.id);
            return [{ ...extra, type: 'toolCall', id: block.id, name: block.name, arguments: block.input }];
          }
          default: throw new Error('Unsupported assistant block');
        }
      });
      for (const id of extras.keys()) if (!msg.blocks.some(b => b.type === 'tool_use' && b.id === id)) throw new Error('Unmatched tool metadata');
      const metadata = msg.blocks.filter(b => b.type === 'provider_data');
      if (metadata.length > 1) throw new Error('Duplicate provider metadata');
      const saved = metadata[0];
      const data = saved?.data;
      if (data) {
        const native = data.native;
        const patches = data.patches;
        if (!Array.isArray(native) || !Array.isArray(patches)) throw new Error('Invalid provider metadata');
        for (const entry of native) {
          if (!entry || !Number.isInteger(entry.index) || entry.index < 0 || entry.index > content.length || !entry.content || typeof entry.content !== 'object') throw new Error('Invalid native content');
          content.splice(entry.index, 0, entry.content);
        }
        for (const patch of patches) {
          if (!patch || !Number.isInteger(patch.index) || !content[patch.index] || !patch.fields || typeof patch.fields !== 'object') throw new Error('Invalid content metadata');
          Object.assign(content[patch.index], patch.fields);
        }
      }
      const stopReason = data?.stopReason;
      if (stopReason !== undefined && stopReason !== 'stop' && stopReason !== 'length' && stopReason !== 'toolUse' && stopReason !== 'error' && stopReason !== 'aborted') throw new Error('Invalid provider stop reason');
      messages.push({ role: 'assistant', content, api: data?.api as AssistantMessage['api'] ?? model.api, provider: saved?.provider ?? model.provider, model: typeof data?.model === 'string' ? data.model : model.id, credentialId: typeof data?.credentialId === 'number' && Number.isSafeInteger(data.credentialId) && data.credentialId > 0 ? data.credentialId : undefined, responseId: typeof data?.responseId === 'string' ? data.responseId : undefined, providerPayload: data?.providerPayload as AssistantMessage['providerPayload'], usage: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, totalTokens: 0, cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 } }, stopReason: stopReason ?? 'stop', timestamp: Date.now() });
    } else if (msg.role === 'user') {
      let content: { type: 'text'; text: string }[] = [];
      const flush = () => { if (content.length) { messages.push({ role: 'user', content, timestamp: Date.now() }); content = []; } };
      for (const block of msg.blocks) {
        if (block.type === 'text') content.push({ type: 'text', text: block.text });
        else if (block.type === 'tool_result') {
          flush();
          const name = toolNames.get(block.tool_use_id);
          if (!name) throw new Error('Unmatched tool result');
          messages.push({ role: 'toolResult', toolCallId: block.tool_use_id, toolName: name, content: [{ type: 'text', text: block.content }], isError: block.is_error, timestamp: Date.now() });
        } else throw new Error('Unsupported user block');
      }
      flush();
    } else throw new Error('Unsupported message role');
  }
  return { systemPrompt: [request.system], messages, tools: request.tools.map(tool => ({ name: tool.name, description: tool.description, parameters: tool.schema })) };
}

export function fromAssistant(message: AssistantMessage) {
  const blocks: Block[] = [];
  const native: { index: number; content: AssistantMessage['content'][number] }[] = [];
  const patches: { index: number; fields: Record<string, unknown> }[] = [];
  for (const [index, item] of message.content.entries()) {
    switch (item.type) {
      case 'text':
        if (item.textSignature) patches.push({ index, fields: { textSignature: item.textSignature } });
        blocks.push({ type: 'text', text: item.text }); break;
      case 'thinking': {
        const { type, thinking, thinkingSignature, ...fields } = item;
        if (Object.keys(fields).length) patches.push({ index, fields });
        blocks.push({ type: 'thinking', thinking, signature: thinkingSignature ?? '' }); break;
      }
      case 'redactedThinking': blocks.push({ type: 'redacted_thinking', data: item.data }); break;
      case 'toolCall': {
        blocks.push({ type: 'tool_use', id: item.id, name: item.name, input: item.arguments });
        const { type, id, name, arguments: args, ...extra } = item;
        if (Object.keys(extra).length) blocks.push({ type: 'tool_extra', tool_use_id: item.id, extra });
        break;
      }
      default: native.push({ index, content: item });
    }
  }
  blocks.push({ type: 'provider_data', provider: message.provider, data: { api: message.api, model: message.model, credentialId: message.credentialId, responseId: message.responseId, providerPayload: message.providerPayload, stopReason: message.stopReason, native, patches } });
  const stop = message.stopReason === 'stop' ? 'end_turn' : message.stopReason === 'length' ? 'max_tokens' : message.stopReason === 'toolUse' ? 'tool_use' : undefined;
  if (!stop) throw new Error('Provider request failed');
  return { message: { role: 'assistant', blocks }, stop, usage: { input_tokens: message.usage.input, output_tokens: message.usage.output, cache_read_tokens: message.usage.cacheRead, cache_write_tokens: message.usage.cacheWrite }, model: message.model };
}
