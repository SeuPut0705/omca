import { expect, test } from 'bun:test';
import { getBundledModel } from '@oh-my-pi/pi-catalog';
import { transformMessages } from '@oh-my-pi/pi-ai/providers/transform-messages';
import { fromAssistant, toContext, type WireRequest } from '../src/messages';

const model = getBundledModel('anthropic', 'claude-sonnet-4-5');
test('tool signatures and result names survive a round trip', () => {
  const request: WireRequest = { system: '', effort: 'medium', max_tokens: 100, tools: [], messages: [
    { role: 'assistant', blocks: [
      { type: 'thinking', thinking: 'reasoning', signature: 'signature' },
      { type: 'tool_use', id: 'call1', name: 'lookup', input: { query: 'x' } },
      { type: 'tool_extra', tool_use_id: 'call1', extra: { thoughtSignature: 'opaque' } },
    ] },
    { role: 'user', blocks: [{ type: 'tool_result', tool_use_id: 'call1', content: 'done', is_error: false }] },
  ] };
  const context = toContext(request, model);
  const assistant = context.messages[0];
  if (assistant.role !== 'assistant') throw new Error('Expected assistant');
  expect(fromAssistant(assistant).message.blocks.filter(block => block.type !== 'provider_data')).toEqual(request.messages[0].blocks.filter(block => block.type !== 'provider_data'));
  expect(context.messages[1]).toMatchObject({ role: 'toolResult', toolCallId: 'call1', toolName: 'lookup', content: [{ type: 'text', text: 'done' }] });
});

test('unmatched tool results fail rather than being dropped', () => {
  const request: WireRequest = { system: '', effort: 'none', max_tokens: 100, tools: [], messages: [{ role: 'user', blocks: [{ type: 'tool_result', tool_use_id: 'missing', content: '', is_error: false }] }] };
  expect(() => toContext(request, model)).toThrow('Unmatched tool result');
});

test('signed text and native content replay with original ordering and response identity', () => {
  const request: WireRequest = { system: '', effort: 'none', max_tokens: 100, tools: [], messages: [{ role: 'assistant', blocks: [{ type: 'text', text: 'answer' }] }] };
  const original = toContext(request, model).messages[0];
  if (original.role !== 'assistant') throw new Error('Expected assistant');
  original.content = [
    { type: 'text', text: 'answer', textSignature: 'signed-text' },
    { type: 'fallback', from: { model: 'a' }, to: { model: 'b' } },
    { type: 'thinking', thinking: 'reason', thinkingSignature: 'signed-reason', itemId: 'item-1' },
  ];
  original.responseId = 'response-1';
  original.credentialId = 41;
  const wire = fromAssistant(original).message;
  const replay = toContext({ ...request, messages: [{ role: 'assistant', blocks: wire.blocks }] }, model).messages[0];
  if (replay.role !== 'assistant') throw new Error('Expected assistant');
  expect(replay.content).toEqual(original.content);
  expect(replay.responseId).toBe('response-1');
  expect(replay.credentialId).toBe(41);
  if (!model.compat || !('supportsContextManagement' in model.compat)) throw new Error('Expected Anthropic model compatibility');
  const signingModel = { ...model, compat: { ...model.compat, signingEndpoint: true } };
  const same = transformMessages([replay], signingModel, undefined, undefined, undefined, signingModel.compat, 41);
  const switched = transformMessages([replay], signingModel, undefined, undefined, undefined, signingModel.compat, 42);
  expect(JSON.stringify(same)).toContain('signed-reason');
  expect(JSON.stringify(switched)).not.toContain('signed-reason');
  const other = getBundledModel('openai', 'gpt-4o');
  const cross = toContext({ ...request, messages: [{ role: 'assistant', blocks: wire.blocks }] }, other).messages[0];
  if (cross.role !== 'assistant') throw new Error('Expected assistant');
  expect(cross.provider).toBe(original.provider);
  expect(cross.api).toBe(original.api);
  expect(cross.model).toBe(original.model);
  expect(cross.credentialId).toBe(41);
  expect(cross.content).toEqual(original.content);
});
