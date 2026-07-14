import assert from 'node:assert/strict';
import { mergeLayoutOverlay } from '../dist/index.js';

const schema = {
  illustration: {
    type: 'object',
    properties: { name: { type: 'string' } },
    $layout: {
      elements: [
        { $ref: '#/illustration/properties/name' },
        { type: 'CustomLayout' },
      ],
    },
  },
};

const resolved = mergeLayoutOverlay(schema, [
  {
    layout_path: '#/illustration/$layout/elements',
    element_idx: 0,
    schema_ref_path: 'illustration.properties.name',
    overlay: { $fullpath: 'illustration.properties.name', $path: 'name', $parentHide: false },
  },
  {
    layout_path: '#/illustration/$layout/elements',
    element_idx: 1,
    schema_ref_path: '',
    overlay: { $fullpath: 'illustration.1', $path: '1', $parentHide: false },
  },
]);

assert.deepEqual(resolved.illustration.properties.name, {
  type: 'string',
  $fullpath: 'illustration.properties.name',
  $path: 'name',
  $parentHide: false,
});
assert.equal(
  resolved.illustration.$layout.elements[1].$fullpath,
  'illustration.$layout.elements.1',
);
