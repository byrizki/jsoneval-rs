import assert from 'node:assert/strict';
import { JSONEvalCore } from '../dist/index.js';

const compactSchema = {
  illustration: {
    type: 'object',
    properties: { name: { type: 'string' } },
    $layout: {
      elements: [
        { $ref: '#/illustration/properties/name' },
        { type: 'TabLayout', elements: [{ $ref: '#/illustration/properties/name' }] },
      ],
    },
  },
};

class JSONEvalWasm {
  getEvaluatedSchemaWithoutParamsJS() {
    return structuredClone(compactSchema);
  }

  getResolvedLayout() {
    return [
      {
        layout_path: '#/illustration/$layout/elements',
        element_idx: 0,
        schema_ref_path: 'illustration.properties.name',
        overlay: {
          $fullpath: 'illustration.properties.name',
          $path: 'name',
          $parentHide: false,
        },
      },
      {
        layout_path: '#/illustration/$layout/elements',
        element_idx: 1,
        schema_ref_path: '',
        overlay: { $fullpath: 'illustration.1', $path: '1', $parentHide: false },
      },
    ];
  }

  getEvaluatedSchemaResolved() {
    throw new Error('wrapper must not call native resolved getter');
  }

  getEvaluatedSchemaResolvedMsgpack() {
    return new Uint8Array([3, 4]);
  }

  getEvaluatedSchemaWithoutParamsSubformJS() {
    return structuredClone({
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
    });
  }

  getResolvedLayoutSubform() {
    return [
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
    ];
  }

  getEvaluatedSchemaResolvedSubform() {
    throw new Error('wrapper must not call native resolved subform getter');
  }
}

const evaluator = new JSONEvalCore(
  { JSONEvalWasm },
  { schema: compactSchema },
);
const resolved = await evaluator.getEvaluatedSchemaResolved();

assert.equal(
  resolved.illustration.$layout.elements[1].$fullpath,
  'illustration.$layout.elements.1',
);
assert.deepEqual(resolved.illustration.properties.name, {
  type: 'string',
  $fullpath: 'illustration.properties.name',
  $path: 'name',
  $parentHide: false,
});

assert.deepEqual(
  await evaluator.getEvaluatedSchemaResolvedMsgpack(),
  new Uint8Array([3, 4]),
);

const resolvedSubform = await evaluator.getEvaluatedSchemaResolvedSubform({
  subformPath: '#/subform',
});
assert.equal(
  resolvedSubform.subform.$layout.elements[1].$fullpath,
  'subform.$layout.elements.1',
);
