#!/usr/bin/env node

/**
 * find_missing_tests.mjs
 *
 * Compares JS csstree test cases against Rust csstree-rs test cases
 * and reports missing test coverage.
 *
 * Usage: node scripts/find_missing_tests.mjs
 */

import { readFileSync, readdirSync, existsSync } from 'fs';
import { join, basename } from 'path';

const JS_TEST_DIR = 'external/csstree/lib/__tests';
const RUST_TEST_DIR = 'tests';
const RUST_SRC_DIR = 'src';

// ─── Extract JS test cases ───

function extractJsTests(filePath) {
  const content = readFileSync(filePath, 'utf8');
  const tests = [];
  const describeStack = [];

  // Track describe() blocks and it() calls
  const lines = content.split('\n');
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    // Match describe('...' or describe("..."
    const describeMatch = line.match(/describe\(\s*['"`](.+?)['"`]/);
    if (describeMatch) {
      describeStack.push(describeMatch[1]);
    }

    // Match closing of describe (approximate — count braces)
    // This is a heuristic, not perfect
    if (line === '});' && describeStack.length > 0) {
      // Could be closing a describe or an it — heuristic
    }

    // Match it('...' or it("..." or it(`...`
    const itMatch = line.match(/\bit\(\s*['"`](.+?)['"`]/);
    if (itMatch) {
      const path = [...describeStack, itMatch[1]].join(' > ');
      tests.push({
        description: itMatch[1],
        path,
        file: basename(filePath),
        line: i + 1,
        type: 'static',
      });
    }

    // Match dynamic test generation patterns
    if (line.includes('forEachTest') || line.includes('forEachAstTest')) {
      tests.push({
        description: `[dynamic] fixture-based tests`,
        path: [...describeStack, '[dynamic fixture tests]'].join(' > '),
        file: basename(filePath),
        line: i + 1,
        type: 'dynamic',
      });
    }

    // Match forEach loops that generate it() blocks
    if (line.includes('.forEach(') && content.substring(
      content.indexOf(line, i > 0 ? lines.slice(0, i).join('\n').length : 0),
      content.indexOf(line, i > 0 ? lines.slice(0, i).join('\n').length : 0) + 500
    ).includes('it(')) {
      tests.push({
        description: `[dynamic] forEach-generated tests`,
        path: [...describeStack, '[forEach tests]'].join(' > '),
        file: basename(filePath),
        line: i + 1,
        type: 'dynamic',
      });
    }
  }

  return tests;
}

// ─── Extract Rust test cases ───

function extractRustTests(filePath) {
  const content = readFileSync(filePath, 'utf8');
  const tests = [];
  const lines = content.split('\n');

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    if (line === '#[test]') {
      // Next non-empty line should be fn name
      for (let j = i + 1; j < lines.length; j++) {
        const fnLine = lines[j].trim();
        const fnMatch = fnLine.match(/fn\s+(\w+)/);
        if (fnMatch) {
          tests.push({
            name: fnMatch[1],
            file: basename(filePath),
            line: j + 1,
          });
          break;
        }
      }
    }
  }

  return tests;
}

// ─── JS file → Rust file mapping ───

const JS_TO_RUST_MAP = {
  'tokenizer.js': ['tokenizer_inline.rs', 'tokenizer_fixtures.rs'],
  'parse.js': ['parser_inline.rs', 'parser_fixtures.rs', 'parse_callbacks_inline.rs'],
  'generate.js': ['generator_inline.rs', 'generator_fixtures.rs'],
  'walk.js': ['walker_inline.rs', 'walker_fixtures.rs', 'walk_extra_inline.rs'],
  'lexer.js': ['lexer_inline.rs', 'lexer_fixtures.rs'],
  'definition-syntax-parse.js': ['definition_syntax_inline.rs', 'definition_syntax_fixtures.rs', 'definition_syntax_extra_inline.rs'],
  'definition-syntax-generate.js': ['definition_syntax_inline.rs', 'definition_syntax_fixtures.rs', 'definition_syntax_extra_inline.rs'],
  'definition-syntax-match.js': ['definition_syntax_match_fixtures.rs', 'definition_syntax_extra_inline.rs'],
  'definition-syntax-walk.js': ['definition_syntax_inline.rs', 'definition_syntax_extra_inline.rs'],
  'find.js': ['walker_inline.rs'],
  'clone.js': [],
  'common.js': ['common_inline.rs'],
  'convert.js': ['convert_inline.rs'],
  'decode-encode.js': ['decode_encode_inline.rs'],
  'exports.js': ['exports_inline.rs'],
  'list.js': ['list_inline.rs'],
  'names.js': ['names_inline.rs'],
  'parse-extension.js': ['parser_inline.rs', 'parse_extension_inline.rs'],
  'parse-selector.js': ['parser_inline.rs', 'parser_fixtures.rs'],
  'nested-selector-disambiguation.js': ['parser_inline.rs'],
  'lexer-match.js': ['lexer_inline.rs', 'lexer_match_extra_inline.rs'],
  'lexer-match-property.js': ['lexer_inline.rs', 'lexer_match_extra_inline.rs'],
  'lexer-match-type.js': ['lexer_inline.rs', 'lexer_match_extra_inline.rs'],
  'lexer-match-result.js': ['lexer_match_extra_inline.rs'],
  'lexer-match-atrule-descriptor.js': ['lexer_atrule_inline.rs'],
  'lexer-match-atrule-prelude.js': ['lexer_atrule_inline.rs'],
  'lexer-match-property-iterations.js': ['lexer_inline.rs', 'lexer_match_extra_inline.rs'],
  'lexer-check-atrule-descriptor.js': ['lexer_atrule_inline.rs'],
  'lexer-check-atrule-name.js': ['lexer_atrule_inline.rs'],
  'lexer-check-atrule-prelude.js': ['lexer_atrule_inline.rs'],
  'lexer-check-property-name.js': ['lexer_inline.rs'],
  'lexer-check-structure.js': ['lexer_structure_inline.rs'],
  'lexer-search-fragments.js': ['lexer_search_inline.rs'],
  'lexer-relative-colors.js': ['lexer_relative_colors_inline.rs'],
};

// ─── Main ───

function main() {
  console.log('='.repeat(80));
  console.log('  CSSTREE-RS TEST GAP ANALYSIS');
  console.log('  Comparing JS csstree tests against Rust csstree-rs tests');
  console.log('='.repeat(80));
  console.log('');

  // Collect all JS tests
  const jsFiles = readdirSync(JS_TEST_DIR).filter(f => f.endsWith('.js'));
  const allJsTests = {};
  let totalJsStatic = 0;
  let totalJsDynamic = 0;

  for (const file of jsFiles) {
    const tests = extractJsTests(join(JS_TEST_DIR, file));
    allJsTests[file] = tests;
    const staticCount = tests.filter(t => t.type === 'static').length;
    const dynamicCount = tests.filter(t => t.type === 'dynamic').length;
    totalJsStatic += staticCount;
    totalJsDynamic += dynamicCount;
  }

  // Collect all Rust tests
  const rustIntegrationTests = {};
  const rustFiles = readdirSync(RUST_TEST_DIR).filter(f => f.endsWith('.rs'));
  let totalRustTests = 0;

  for (const file of rustFiles) {
    const tests = extractRustTests(join(RUST_TEST_DIR, file));
    rustIntegrationTests[file] = tests;
    totalRustTests += tests.length;
  }

  // Also collect unit tests from src/
  const rustUnitTests = [];
  function scanRustDir(dir) {
    try {
      const entries = readdirSync(dir, { withFileTypes: true });
      for (const entry of entries) {
        const path = join(dir, entry.name);
        if (entry.isDirectory()) {
          scanRustDir(path);
        } else if (entry.name.endsWith('.rs')) {
          const tests = extractRustTests(path);
          if (tests.length > 0) {
            rustUnitTests.push(...tests.map(t => ({ ...t, file: path.replace(RUST_SRC_DIR + '/', '') })));
          }
        }
      }
    } catch (e) { /* ignore */ }
  }
  scanRustDir(RUST_SRC_DIR);

  // ─── Report ───

  console.log('SUMMARY');
  console.log('─'.repeat(60));
  console.log(`JS test files:           ${jsFiles.length}`);
  console.log(`JS static it() blocks:   ${totalJsStatic}`);
  console.log(`JS dynamic test sources: ${totalJsDynamic}`);
  console.log(`Rust integration tests:  ${totalRustTests}`);
  console.log(`Rust unit tests:         ${rustUnitTests.length}`);
  console.log('');

  // Per-file comparison
  console.log('');
  console.log('PER-FILE GAP ANALYSIS');
  console.log('─'.repeat(60));

  const unmappedJsFiles = [];
  const missingTests = [];
  let totalMissing = 0;

  for (const jsFile of jsFiles.sort()) {
    const jsTests = allJsTests[jsFile] || [];
    const staticTests = jsTests.filter(t => t.type === 'static');
    const dynamicTests = jsTests.filter(t => t.type === 'dynamic');
    const rustFiles = JS_TO_RUST_MAP[jsFile];

    if (!rustFiles) {
      unmappedJsFiles.push(jsFile);
      continue;
    }

    if (rustFiles.length === 0) {
      if (staticTests.length > 0 || dynamicTests.length > 0) {
        console.log(`\n[MISSING] ${jsFile} (${staticTests.length} static, ${dynamicTests.length} dynamic) → NO RUST EQUIVALENT`);
        for (const t of staticTests) {
          console.log(`  - ${t.description}`);
          missingTests.push({ jsFile, test: t.description, reason: 'no_rust_file' });
          totalMissing++;
        }
      }
      continue;
    }

    // Gather Rust test names from mapped files
    const rustTestNames = new Set();
    for (const rf of rustFiles) {
      const tests = rustIntegrationTests[rf] || [];
      for (const t of tests) {
        rustTestNames.add(t.name.toLowerCase());
      }
    }

    // Try to match JS tests to Rust tests (fuzzy)
    const unmatched = [];
    for (const jt of staticTests) {
      const desc = jt.description.toLowerCase()
        .replace(/[^a-z0-9]/g, '_')
        .replace(/_+/g, '_')
        .replace(/^_|_$/g, '');

      // Try various matching strategies
      let found = false;
      for (const rn of rustTestNames) {
        if (rn.includes(desc) || desc.includes(rn)) {
          found = true;
          break;
        }
        // Try matching key words
        const descWords = desc.split('_').filter(w => w.length > 2);
        const rnWords = rn.split('_').filter(w => w.length > 2);
        const overlap = descWords.filter(w => rnWords.includes(w));
        if (overlap.length >= 2 && overlap.length >= descWords.length * 0.5) {
          found = true;
          break;
        }
      }
      if (!found) {
        unmatched.push(jt);
      }
    }

    if (unmatched.length > 0 || dynamicTests.length > 0) {
      const mapped = rustFiles.join(', ');
      console.log(`\n${jsFile} → ${mapped}`);
      console.log(`  JS: ${staticTests.length} static, ${dynamicTests.length} dynamic | Rust: ${rustTestNames.size} tests`);

      if (unmatched.length > 0) {
        console.log(`  UNMATCHED JS TESTS (${unmatched.length}):`);
        for (const t of unmatched) {
          console.log(`    - ${t.description} (line ${t.line})`);
          missingTests.push({ jsFile, test: t.description, reason: 'no_match' });
          totalMissing++;
        }
      }

      if (dynamicTests.length > 0) {
        console.log(`  DYNAMIC TESTS (${dynamicTests.length} sources — may generate many individual tests):`);
        for (const t of dynamicTests) {
          console.log(`    - ${t.description} (line ${t.line})`);
        }
      }
    }
  }

  if (unmappedJsFiles.length > 0) {
    console.log('\n\nUNMAPPED JS FILES (no mapping defined):');
    for (const f of unmappedJsFiles) {
      const tests = allJsTests[f] || [];
      console.log(`  ${f}: ${tests.filter(t => t.type === 'static').length} static tests`);
    }
  }

  // ─── Final Summary ───

  console.log('\n');
  console.log('='.repeat(80));
  console.log('  FINAL SUMMARY');
  console.log('='.repeat(80));
  console.log(`Total unmatched static JS tests: ${totalMissing}`);
  console.log(`Total JS files with no Rust equivalent: ${unmappedJsFiles.length}`);
  console.log('');

  // Group missing by category
  const byReason = {};
  for (const m of missingTests) {
    const key = m.reason === 'no_rust_file' ? `[NO RUST FILE] ${m.jsFile}` : m.jsFile;
    byReason[key] = byReason[key] || [];
    byReason[key].push(m.test);
  }

  if (Object.keys(byReason).length > 0) {
    console.log('MISSING TESTS BY FILE:');
    for (const [key, tests] of Object.entries(byReason).sort()) {
      console.log(`\n  ${key} (${tests.length}):`);
      for (const t of tests) {
        console.log(`    - ${t}`);
      }
    }
  }
}

main();
