jest.mock('react-native', () => ({
  NativeModules: {
    JsonEvalRs: {
      create: jest.fn(() => 'handle'),
      getEvaluatedSchemaWithoutParams: jest.fn(() => JSON.stringify({
        illustration: {
          type: 'object',
          properties: { name: { type: 'string' } },
          $layout: {
            elements: [
              { $ref: '#/illustration/properties/name' },
              { type: 'TabLayout' },
            ],
          },
        },
      })),
      getResolvedLayout: jest.fn(() => JSON.stringify([
        {
          layout_path: '#/illustration/$layout/elements',
          element_idx: 0,
          schema_ref_path: 'illustration.properties.name',
          overlay: {},
        },
        {
          layout_path: '#/illustration/$layout/elements',
          element_idx: 1,
          schema_ref_path: '',
          overlay: { $fullpath: 'illustration.1', $path: '1', $parentHide: false },
        },
      ])),
      getEvaluatedSchemaResolved: jest.fn(() => {
        throw new Error('wrapper must not call native resolved getter');
      }),
      getEvaluatedSchemaMsgpack: jest.fn(() => [1, 2]),
      getEvaluatedSchemaResolvedMsgpack: jest.fn(() => [3, 4]),
      getEvaluatedSchemaWithoutParamsSubform: jest.fn(() => JSON.stringify({
        subform: {
          type: 'object',
          properties: { name: { type: 'string' } },
          $layout: {
            elements: [
              { $ref: '#/subform/properties/name' },
              { type: 'TabLayout' },
            ],
          },
        },
      })),
      getResolvedLayoutSubform: jest.fn(() => JSON.stringify([
        {
          layout_path: '#/subform/$layout/elements',
          element_idx: 0,
          schema_ref_path: 'subform.properties.name',
          overlay: {},
        },
        {
          layout_path: '#/subform/$layout/elements',
          element_idx: 1,
          schema_ref_path: '',
          overlay: { $fullpath: 'subform.1', $path: '1', $parentHide: false },
        },
      ])),
      getEvaluatedSchemaResolvedSubform: jest.fn(() => {
        throw new Error('wrapper must not call native resolved subform getter');
      }),
    },
  },
  Platform: { select: jest.fn(() => '') },
}));

import { JSONEval } from '../index';

describe('resolved schema composition', () => {
  const evaluator = new JSONEval({ schema: {} });

  it('composes root compact schema and overlays without native resolved getter', async () => {
    const resolved = await evaluator.getEvaluatedSchemaResolved();

    expect(resolved.illustration.$layout.elements[1].$fullpath).toBe(
      'illustration.$layout.elements.1',
    );
    expect(resolved.illustration.properties.name).toMatchObject({
      $fullpath: 'illustration.properties.name',
      $path: 'name',
      $parentHide: false,
    });
  });

  it('forwards compact and resolved MessagePack bytes from native', async () => {
    await expect(evaluator.getEvaluatedSchemaMsgpack()).resolves.toEqual(
      new Uint8Array([1, 2]),
    );
    await expect(evaluator.getEvaluatedSchemaResolvedMsgpack()).resolves.toEqual(
      new Uint8Array([3, 4]),
    );
  });

  it('composes subform compact schema and overlays without native resolved getter', async () => {
    const resolved = await evaluator.getEvaluatedSchemaResolvedSubform({
      subformPath: '#/subform',
    });

    expect(resolved.subform.$layout.elements[1].$fullpath).toBe(
      'subform.$layout.elements.1',
    );
  });
});
