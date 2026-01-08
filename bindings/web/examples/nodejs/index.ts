import { JSONEval } from "@json-eval-rs/node";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import assert from "node:assert";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

test("Node.js Binding Example Test", async (t) => {
  // Read schema
  const schemaPath = path.resolve(
    __dirname,
    "../../../../tests/fixtures/minimal_form.json"
  );
  console.log("Reading schema from:", schemaPath);
  const schemaStr = fs.readFileSync(schemaPath, "utf8");

  // Read data
  const dataPath = path.resolve(__dirname, "data.json");
  console.log("Reading data from:", dataPath);
  const dataStr = fs.readFileSync(dataPath, "utf8");

  console.log("Validating JSON...");
  const schema = JSON.parse(schemaStr);
  const data = JSON.parse(dataStr);
  
  assert.doesNotThrow(() => JSON.parse(schemaStr), "Schema should be valid JSON");
  assert.doesNotThrow(() => JSON.parse(dataStr), "Data should be valid JSON");

  console.log("Creating JSONEval instance...");
  const je = new JSONEval({
    schema,
    data,
    context: {},
  });

  console.log("Calling evaluate()...");
  await je.evaluate({ data });

  console.log("Calling getEvaluatedSchema()...");
  const evaluatedSchema = await je.getEvaluatedSchema({ skipLayout: false });

  console.log("Result type:", typeof evaluatedSchema);
  
  assert.ok(evaluatedSchema, "Evaluated schema should not be null or undefined");
  
  if (evaluatedSchema instanceof Map) {
    console.log("Result is a Map! Keys:", Array.from(evaluatedSchema.keys()));
    console.log("Map Size:", evaluatedSchema.size);
    assert.ok(evaluatedSchema.size > 0, "Evaluated schema Map should not be empty");
  } else {
    const keys = Object.keys(evaluatedSchema as object);
    console.log("Result Object keys:", keys.length);
    console.log("First few keys:", keys.slice(0, 5));
    assert.ok(keys.length > 0, "Evaluated schema Object should not be empty");
  }
});
