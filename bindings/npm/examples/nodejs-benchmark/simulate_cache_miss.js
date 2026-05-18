const fs = require('fs');
const path = require('path');
const { JSONEvalWasm } = require('../../packages/node/pkg/json_eval_rs.js');

async function main() {
    console.log("Loading files...");
    const schemaText = fs.readFileSync(path.join(__dirname, '../../../samples/zpp.json'), 'utf8');
    const dataText = fs.readFileSync(path.join(__dirname, '../../../samples/zpp-data.json'), 'utf8');
    
    const schemaStr = JSON.stringify(JSON.parse(schemaText));
    const dataStr = JSON.stringify(JSON.parse(dataText));
    const contextStr = JSON.stringify({});

    console.log("Starting eval...");
    const je = new JSONEvalWasm(schemaStr, contextStr, dataStr);

    console.log("\n--- Message 1 (First Tick) ---");
    let t = performance.now();
    je.evaluate(dataStr, contextStr, null);
    console.log(`Eval 1: ${(performance.now() - t).toFixed(2)}ms`);

    t = performance.now();
    je.evaluate(dataStr, contextStr, null);
    console.log(`Eval 2: ${(performance.now() - t).toFixed(2)}ms`);

    console.log("\n--- Simulating delay between Worker messages ---");
    await new Promise(r => setTimeout(r, 500));

    console.log("\n--- Message 2 (Second Tick) ---");
    // Simulate user reading data from message and parsing/stringifying again
    const newDataStr = JSON.stringify(JSON.parse(dataText));
    const newContextStr = JSON.stringify({});

    t = performance.now();
    je.evaluate(newDataStr, newContextStr, null);
    console.log(`Eval 3: ${(performance.now() - t).toFixed(2)}ms`);

    t = performance.now();
    je.evaluate(newDataStr, newContextStr, null);
    console.log(`Eval 4: ${(performance.now() - t).toFixed(2)}ms`);
}

main().catch(console.error);
