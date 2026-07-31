import assert from 'node:assert/strict';
import {
  mergeLayoutOverlay,
  parseJsonWithBigInt,
  parseValue,
  resolveEvaluatedLayout,
  stringifyJsonWithBigInt,
  stringifyValue,
} from '../dist/index.js';

const unsafeInteger = 1000000000000000000n;

assert.deepEqual(
  parseJsonWithBigInt('{"safe":9007199254740991,"unsafe":1000000000000000000,"negative":-9007199254740992,"decimal":1.5,"exponent":1e18,"text":"1000000000000000000"}'),
  {
    safe: 9007199254740991,
    unsafe: unsafeInteger,
    negative: -9007199254740992n,
    decimal: 1.5,
    exponent: 1e18,
    text: '1000000000000000000',
  },
  'unsafe integer tokens must parse as bigint without changing strings, decimals, or exponents',
);

const bigintJson = stringifyJsonWithBigInt({
  unsafe: unsafeInteger,
  nested: [-9007199254740992n, '1000000000000000000'],
});
assert.equal(
  bigintJson,
  '{"unsafe":1000000000000000000,"nested":[-9007199254740992,"1000000000000000000"]}',
  'bigint values must serialize as unquoted JSON integers',
);
assert.deepEqual(
  parseValue(bigintJson),
  { unsafe: unsafeInteger, nested: [-9007199254740992n, '1000000000000000000'] },
  'parseValue must preserve bigint values emitted by stringifyJsonWithBigInt',
);
assert.equal(
  stringifyValue({ unsafe: unsafeInteger }),
  '{"unsafe":1000000000000000000}',
  'stringifyValue must support bigint input',
);

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

for (const unsafeKey of ['__proto__', 'constructor', 'prototype']) {
  const maliciousRefSchema = {
    layout: {
      $layout: {
        elements: [{ $ref: `#/${unsafeKey}` }],
      },
    },
  };

  mergeLayoutOverlay(maliciousRefSchema, [{
    layout_path: '#/layout/$layout/elements',
    element_idx: 0,
    schema_ref_path: unsafeKey,
    overlay: {},
  }]);

  assert.equal(
    maliciousRefSchema.layout.$layout.elements[0].$ref,
    `#/${unsafeKey}`,
    `${unsafeKey} must not resolve through the JavaScript prototype chain`,
  );

  const maliciousOverlaySchema = JSON.parse('{"layout":{"$layout":{"elements":[{}]}}}');
  mergeLayoutOverlay(maliciousOverlaySchema, [{
    layout_path: '#/layout/$layout/elements',
    element_idx: 0,
    schema_ref_path: '',
    overlay: JSON.parse(`{"${unsafeKey}":{"polluted":true}}`),
  }]);

  const target = maliciousOverlaySchema.layout.$layout.elements[0];
  assert.equal(
    Object.hasOwn(target, unsafeKey),
    false,
    `layout overlays must not assign ${unsafeKey}`,
  );
  assert.equal(
    Object.getPrototypeOf(target),
    Object.prototype,
    `layout overlays must not mutate the target prototype through ${unsafeKey}`,
  );
}
