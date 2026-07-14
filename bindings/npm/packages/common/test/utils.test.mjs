import assert from 'node:assert/strict';
import { resolveEvaluatedLayout } from '../dist/index.js';

const schema = {
  $params: { internal: true },
  illustration: {
    type: 'object',
    properties: { name: { type: 'string' } },
    $layout: {
      elements: [
        { $ref: '#/illustration/properties/name' },
        {
          type: 'TabLayout',
          elements: [{ $ref: '#/illustration/properties/name' }],
        },
      ],
    },
  },
};

const resolved = await resolveEvaluatedLayout(
  async () => structuredClone(schema),
  async () => [
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
  ],
);

assert.equal(resolved.$params.internal, true, 'helper leaves caller-selected compact schema unchanged');
assert.deepEqual(resolved.illustration.properties.name, {
  type: 'string',
  $fullpath: 'illustration.properties.name',
  $path: 'name',
  $parentHide: false,
});
assert.equal(
  resolved.illustration.$layout.elements[0].$fullpath,
  'illustration.properties.name',
  'resolved $ref item must retain target schema path',
);
assert.equal(
  resolved.illustration.$layout.elements[1].$fullpath,
  'illustration.$layout.elements.1',
  'inline TabLayout must use literal structural schema path',
);

const resolvedRefWithoutMetadataOverlay = await resolveEvaluatedLayout(
  async () => ({
    illustration: {
      properties: { name: { type: 'string' } },
      $layout: { elements: [{ $ref: '#/illustration/properties/name' }] },
    },
  }),
  async () => [{
    layout_path: '#/illustration/$layout/elements',
    element_idx: 0,
    schema_ref_path: 'illustration.properties.name',
    overlay: {},
  }],
);

assert.equal(
  resolvedRefWithoutMetadataOverlay.illustration.$layout.elements[0].$fullpath,
  'illustration.properties.name',
  'resolved $ref must retain target path after final recursive stamping',
);

const resolvedNonPropertyRef = await resolveEvaluatedLayout(
  async () => ({
    illustration: {
      definition: { type: 'string' },
      $layout: { elements: [{ $ref: '#/illustration/definition' }] },
    },
  }),
  async () => [{
    layout_path: '#/illustration/$layout/elements',
    element_idx: 0,
    schema_ref_path: 'illustration.definition',
    overlay: {},
  }],
);

assert.equal(
  resolvedNonPropertyRef.illustration.$layout.elements[0].$fullpath,
  'illustration.definition',
  'resolved non-property $ref must retain full target path',
);

const resolvedWithoutLayout = await resolveEvaluatedLayout(
  async () => ({
    profile: {
      type: 'object',
      properties: { email: { type: 'string' } },
    },
  }),
  async () => [],
);

assert.deepEqual(
  resolvedWithoutLayout.profile.properties.email,
  {
    type: 'string',
    $fullpath: 'profile.properties.email',
    $path: 'email',
    $parentHide: false,
  },
  'resolver must preserve native property metadata even when schema has no layouts',
);
