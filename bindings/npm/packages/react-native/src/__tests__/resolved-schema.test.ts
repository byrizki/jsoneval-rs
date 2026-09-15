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
      getEvaluatedSchema: jest.fn(() => '{"limit":1000000000000000000}'),
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
      getPlainParams: jest.fn(() => JSON.stringify({ meta: 'v1' })),
      getEvaluatedParams: jest.fn((_handle, withStaticArray) =>
        JSON.stringify(
          withStaticArray
            ? { meta: 'v1', static_data: [1, 2, 3] }
            : { meta: 'v1' }
        )
      ),
      getPlainParamsSubform: jest.fn(() => JSON.stringify({ sub_meta: 'sub_v1' })),
      getEvaluatedParamsSubform: jest.fn((_handle, _subformPath, withStaticArray) =>
        JSON.stringify(
          withStaticArray
            ? { sub_meta: 'sub_v1', sub_static: [1, 2] }
            : { sub_meta: 'sub_v1' }
        )
      ),
      getSchemaValue: jest.fn((_handle, includeSubforms) =>
        JSON.stringify({
          data: { total: includeSubforms ? 50 : null },
        })
      ),
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

  it('parses unsafe JSON integers as bigint from native results', async () => {
    await expect(evaluator.getEvaluatedSchema()).resolves.toEqual({
      limit: 1000000000000000000n,
    });
  });

  it('composes subform compact schema and overlays without native resolved getter', async () => {
    const resolved = await evaluator.getEvaluatedSchemaResolvedSubform({
      subformPath: '#/subform',
    });

    expect(resolved.subform.$layout.elements[1].$fullpath).toBe(
      'subform.$layout.elements.1',
    );
  });

  it('retrieves plain and evaluated params with static array options', async () => {
    const plain = await evaluator.getPlainParams();
    expect(plain).toEqual({ meta: 'v1' });

    const evalWithout = await evaluator.getEvaluatedParams(false);
    expect(evalWithout).toEqual({ meta: 'v1' });

    const evalWith = await evaluator.getEvaluatedParams(true);
    expect(evalWith).toEqual({ meta: 'v1', static_data: [1, 2, 3] });
  });

  it('retrieves subform plain and evaluated params', async () => {
    const plainSub = await evaluator.getPlainParamsSubform({
      subformPath: '#/subform',
    });
    expect(plainSub).toEqual({ sub_meta: 'sub_v1' });

    const evalSubWithout = await evaluator.getEvaluatedParamsSubform({
      subformPath: '#/subform',
      withStaticArray: false,
    });
    expect(evalSubWithout).toEqual({ sub_meta: 'sub_v1' });

    const evalSubWith = await evaluator.getEvaluatedParamsSubform({
      subformPath: '#/subform',
      withStaticArray: true,
    });
    expect(evalSubWith).toEqual({ sub_meta: 'sub_v1', sub_static: [1, 2] });
  });

  it('forwards includeSubforms flag in getSchemaValue', async () => {
    const valDefault = await evaluator.getSchemaValue();
    expect(valDefault).toEqual({ data: { total: null } });

    const valWithSubforms = await evaluator.getSchemaValue(true);
    expect(valWithSubforms).toEqual({ data: { total: 50 } });
  });
});
