/**
 * Syncs test cases from external/csstree into tests/fixtures/ and generates
 * tests/test_manifest.json as the canonical list of every test case.
 *
 * Run: node scripts/extract_test_manifest.mjs
 *
 * This script is idempotent and incremental:
 * - Re-running it picks up added/changed/removed test cases from the JS source
 * - It copies fixture JSON files into tests/fixtures/ (our owned copy)
 * - It extracts inline JS test cases into tests/fixtures/inline/*.json
 * - It generates tests/test_manifest.json listing every test case by ID
 *
 * The manifest structure:
 * {
 *   "test_cases": [
 *     {
 *       "id": "fixture::tokenize/ident-token.json::valid[0]",
 *       "module": "tokenizer",
 *       "category": "fixture",
 *       "fixture_file": "tokenize/ident-token.json",
 *       "input": "foo",
 *       "kind": "valid"
 *     },
 *     {
 *       "id": "inline::parse::browser hacks > should parse *property",
 *       "module": "parser",
 *       "category": "inline",
 *       "source_file": "parse.js",
 *       "describe": "browser hacks",
 *       "it": "should parse *property as Declaration (browser hack)",
 *       "css_input": "html { *zoom: 1; }"
 *     },
 *     ...
 *   ]
 * }
 */

import { readFileSync, writeFileSync, readdirSync, statSync, existsSync, mkdirSync, cpSync } from 'fs';
import { join, relative, basename, extname, dirname } from 'path';

const ROOT = new URL('..', import.meta.url).pathname.replace(/\/$/, '');
const JS_ROOT = join(ROOT, 'external/csstree');
const JS_FIXTURES = join(JS_ROOT, 'fixtures');
const JS_TESTS = join(JS_ROOT, 'lib/__tests');
const OUR_FIXTURES = join(ROOT, 'tests/fixtures');
const OUR_INLINE = join(ROOT, 'tests/fixtures/inline');
const MANIFEST_PATH = join(ROOT, 'tests/test_manifest.json');

// ── Ensure directories ──
mkdirSync(OUR_FIXTURES, { recursive: true });
mkdirSync(OUR_INLINE, { recursive: true });

const testCases = [];
let stats = { copied: 0, extracted: 0, total: 0 };

// ── Helper: walk directory ──
function walkDir(dir) {
  const results = [];
  if (!existsSync(dir)) return results;
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    const stat = statSync(full);
    if (stat.isDirectory()) results.push(...walkDir(full));
    else results.push(full);
  }
  return results.sort();
}

// ── Helper: module mapping ──
function fixtureToModule(fixturePath) {
  if (fixturePath.startsWith('tokenize/')) return 'tokenizer';
  if (fixturePath.startsWith('ast/')) return 'parser';
  if (fixturePath.startsWith('definition-syntax-match/')) return 'lexer';
  if (fixturePath.startsWith('definition-syntax/')) return 'definition_syntax';
  return 'unknown';
}

function jsTestFileToModule(filename) {
  const name = basename(filename, '.js');
  if (name === 'tokenizer') return 'tokenizer';
  if (name.startsWith('parse') || name === 'nested-selector-disambiguation') return 'parser';
  if (name === 'generate') return 'generator';
  if (name.startsWith('walk') || name === 'find') return 'walker';
  if (name.startsWith('lexer')) return 'lexer';
  if (name.startsWith('definition-syntax')) return 'definition_syntax';
  if (['list', 'clone', 'convert', 'decode-encode', 'names'].includes(name)) return 'utils';
  if (['common', 'exports'].includes(name)) return 'common';
  return 'unknown';
}

// ══════════════════════════════════════════════════════════════
// PHASE 1: Copy fixture JSON files and extract test case IDs
// ══════════════════════════════════════════════════════════════

console.log('Phase 1: Syncing fixture files...\n');

// ── Tokenize fixtures ──
for (const file of walkDir(join(JS_FIXTURES, 'tokenize'))) {
  const rel = relative(JS_FIXTURES, file);
  const dest = join(OUR_FIXTURES, rel);
  mkdirSync(dirname(dest), { recursive: true });
  cpSync(file, dest);
  stats.copied++;

  const data = JSON.parse(readFileSync(file, 'utf8'));
  const valid = data.valid || [];
  const invalid = data.invalid || [];

  valid.forEach((v, i) => {
    const input = typeof v === 'string' ? v : v.value;
    testCases.push({
      id: `fixture::${rel}::valid[${i}]`,
      module: 'tokenizer',
      category: 'fixture',
      fixture_file: rel,
      input,
      kind: 'valid',
    });
  });

  invalid.forEach((v, i) => {
    const input = typeof v === 'string' ? v : v.value;
    testCases.push({
      id: `fixture::${rel}::invalid[${i}]`,
      module: 'tokenizer',
      category: 'fixture',
      fixture_file: rel,
      input,
      kind: 'invalid',
    });
  });
}

// ── AST fixtures ──
for (const file of walkDir(join(JS_FIXTURES, 'ast'))) {
  if (extname(file) !== '.json') continue;
  const rel = relative(JS_FIXTURES, file);
  const dest = join(OUR_FIXTURES, rel);
  mkdirSync(dirname(dest), { recursive: true });
  cpSync(file, dest);
  stats.copied++;

  const data = JSON.parse(readFileSync(file, 'utf8'));
  if (typeof data === 'object' && !Array.isArray(data)) {
    for (const [name, test] of Object.entries(data)) {
      testCases.push({
        id: `fixture::${rel}::${name}`,
        module: 'parser',
        category: 'fixture',
        fixture_file: rel,
        test_name: name,
        css_input: test.source || null,
        expected_generate: test.generate || null,
        error: test.error || null,
        options: test.options || null,
      });
    }
  }
}

// ── Definition-syntax-match fixtures ──
for (const file of walkDir(join(JS_FIXTURES, 'definition-syntax-match'))) {
  if (extname(file) !== '.json') continue;
  const rel = relative(JS_FIXTURES, file);
  const dest = join(OUR_FIXTURES, rel);
  mkdirSync(dirname(dest), { recursive: true });
  cpSync(file, dest);
  stats.copied++;

  const data = JSON.parse(readFileSync(file, 'utf8'));
  for (const [groupName, group] of Object.entries(data)) {
    (group.valid || []).forEach((v, i) => {
      testCases.push({
        id: `fixture::${rel}::${groupName}::valid[${i}]`,
        module: 'lexer',
        category: 'fixture',
        fixture_file: rel,
        group: groupName,
        input: v,
        kind: 'valid',
      });
    });
    (group.invalid || []).forEach((v, i) => {
      testCases.push({
        id: `fixture::${rel}::${groupName}::invalid[${i}]`,
        module: 'lexer',
        category: 'fixture',
        fixture_file: rel,
        group: groupName,
        input: v,
        kind: 'invalid',
      });
    });
    if (group.matchResult) {
      for (const [input] of Object.entries(group.matchResult)) {
        testCases.push({
          id: `fixture::${rel}::${groupName}::matchResult::${input}`,
          module: 'lexer',
          category: 'fixture',
          fixture_file: rel,
          group: groupName,
          input,
          kind: 'matchResult',
        });
      }
    }
  }
}

// ── Definition-syntax fixtures ──
for (const file of walkDir(join(JS_FIXTURES, 'definition-syntax'))) {
  if (extname(file) !== '.json') continue;
  const rel = relative(JS_FIXTURES, file);
  const dest = join(OUR_FIXTURES, rel);
  mkdirSync(dirname(dest), { recursive: true });
  cpSync(file, dest);
  stats.copied++;

  const data = JSON.parse(readFileSync(file, 'utf8'));
  for (const [groupName, group] of Object.entries(data)) {
    if (typeof group !== 'object' || group === null) continue;
    (group.valid || []).forEach((v, i) => {
      testCases.push({
        id: `fixture::${rel}::${groupName}::valid[${i}]`,
        module: 'definition_syntax',
        category: 'fixture',
        fixture_file: rel,
        group: groupName,
        input: typeof v === 'string' ? v : JSON.stringify(v),
        kind: 'valid',
      });
    });
    (group.invalid || []).forEach((v, i) => {
      testCases.push({
        id: `fixture::${rel}::${groupName}::invalid[${i}]`,
        module: 'definition_syntax',
        category: 'fixture',
        fixture_file: rel,
        group: groupName,
        input: typeof v === 'string' ? v : JSON.stringify(v),
        kind: 'invalid',
      });
    });
  }
}

// ── Source maps fixtures ──
for (const file of walkDir(join(JS_FIXTURES, 'sourceMaps'))) {
  const rel = relative(JS_FIXTURES, file);
  const dest = join(OUR_FIXTURES, rel);
  mkdirSync(dirname(dest), { recursive: true });
  cpSync(file, dest);
  stats.copied++;
}

// ── stringify fixtures ──
for (const name of ['stringify.css', 'stringify.ast']) {
  const src = join(JS_FIXTURES, name);
  if (existsSync(src)) {
    cpSync(src, join(OUR_FIXTURES, name));
    stats.copied++;
  }
}

// ══════════════════════════════════════════════════════════════
// PHASE 2: Extract inline test cases from JS test files
// ══════════════════════════════════════════════════════════════

console.log('Phase 2: Extracting inline test cases...\n');

const testFiles = readdirSync(JS_TESTS)
  .filter(f => f.endsWith('.js') && !f.startsWith('.'))
  .sort();

for (const file of testFiles) {
  const filePath = join(JS_TESTS, file);
  if (statSync(filePath).isDirectory()) continue;

  const content = readFileSync(filePath, 'utf8');
  const moduleName = basename(file, '.js');
  const module = jsTestFileToModule(file);
  const inlineTests = [];

  // Extract describe/it structure
  const lines = content.split('\n');
  const describeStack = [];
  let braceDepth = 0;
  const describeDepths = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Track brace depth for describe scope
    for (const ch of trimmed) {
      if (ch === '{') braceDepth++;
      if (ch === '}') {
        braceDepth--;
        // Pop describe stack when we close its scope
        while (describeDepths.length > 0 && describeDepths[describeDepths.length - 1] >= braceDepth) {
          describeDepths.pop();
          describeStack.pop();
        }
      }
    }

    // Match describe
    const describeMatch = trimmed.match(/^(?:describe|context)\(\s*(['"`])((?:[^\\]|\\.)*?)\1/);
    if (describeMatch) {
      describeStack.push(describeMatch[2]);
      describeDepths.push(braceDepth - 1); // -1 because we already counted the opening {
    }

    // Match it()
    const itMatch = trimmed.match(/^it\(\s*(['"`])((?:[^\\]|\\.)*?)\1/);
    if (itMatch) {
      const cssInput = extractCssInput(lines, i);
      const describe = describeStack.join(' > ');
      const testCase = {
        describe,
        it: itMatch[2],
        css_input: cssInput,
        line: i + 1,
      };
      inlineTests.push(testCase);

      testCases.push({
        id: `inline::${moduleName}::${describe ? describe + ' > ' : ''}${itMatch[2]}`,
        module,
        category: 'inline',
        source_file: file,
        describe,
        it: itMatch[2],
        css_input: cssInput,
        line: i + 1,
      });
    }
  }

  if (inlineTests.length > 0) {
    // Write inline test data to our fixtures
    writeFileSync(
      join(OUR_INLINE, `${moduleName}.json`),
      JSON.stringify(inlineTests, null, 2)
    );
    stats.extracted += inlineTests.length;
  }
}

// ══════════════════════════════════════════════════════════════
// PHASE 3: Write manifest
// ══════════════════════════════════════════════════════════════

stats.total = testCases.length;

const manifest = {
  _generated: new Date().toISOString(),
  _description: 'Canonical list of all test cases from external/csstree. Regenerate with: node scripts/extract_test_manifest.mjs',
  _stats: {
    fixture_files_copied: stats.copied,
    inline_tests_extracted: stats.extracted,
    total_test_cases: stats.total,
  },
  test_cases: testCases,
};

writeFileSync(MANIFEST_PATH, JSON.stringify(manifest, null, 2));

// ══════════════════════════════════════════════════════════════
// Summary
// ══════════════════════════════════════════════════════════════

const byModule = {};
const byCategory = {};
for (const tc of testCases) {
  byModule[tc.module] = (byModule[tc.module] || 0) + 1;
  byCategory[tc.category] = (byCategory[tc.category] || 0) + 1;
}

console.log('=== Sync Complete ===\n');
console.log(`Fixture files copied:    ${stats.copied}`);
console.log(`Inline tests extracted:  ${stats.extracted}`);
console.log(`Total test cases:        ${stats.total}\n`);

console.log('By module:');
for (const [mod, count] of Object.entries(byModule).sort()) {
  console.log(`  ${mod.padEnd(20)} ${count}`);
}

console.log('\nBy category:');
for (const [cat, count] of Object.entries(byCategory).sort()) {
  console.log(`  ${cat.padEnd(20)} ${count}`);
}

console.log(`\nManifest: ${MANIFEST_PATH}`);
console.log(`Fixtures: ${OUR_FIXTURES}/`);

// ── CSS input extraction helper ──
function extractCssInput(lines, itLine) {
  for (let j = itLine; j < Math.min(itLine + 20, lines.length); j++) {
    const line = lines[j].trim();
    // Match parse('...')
    const parseMatch = line.match(/parse\(\s*(['"`])((?:[^\\]|\\.)*?)\1/);
    if (parseMatch) return unescapeJS(parseMatch[2]);
    // Match source: '...'
    const sourceMatch = line.match(/source:\s*(['"`])((?:[^\\]|\\.)*?)\1/);
    if (sourceMatch) return unescapeJS(sourceMatch[2]);
    // Match matchProperty('prop', '...')
    const matchPropMatch = line.match(/matchProperty\(\s*['"`].*?['"`]\s*,\s*(['"`])((?:[^\\]|\\.)*?)\1/);
    if (matchPropMatch) return unescapeJS(matchPropMatch[2]);
    // Match matchType('type', '...')
    const matchTypeMatch = line.match(/matchType\(\s*['"`].*?['"`]\s*,\s*(['"`])((?:[^\\]|\\.)*?)\1/);
    if (matchTypeMatch) return unescapeJS(matchTypeMatch[2]);
  }
  return null;
}

function unescapeJS(s) {
  return s.replace(/\\n/g, '\n').replace(/\\t/g, '\t').replace(/\\r/g, '\r').replace(/\\'/g, "'").replace(/\\"/g, '"').replace(/\\\\/g, '\\');
}
