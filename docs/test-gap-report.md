================================================================================
  CSSTREE-RS TEST GAP ANALYSIS
  Comparing JS csstree tests against Rust csstree-rs tests
================================================================================

SUMMARY
────────────────────────────────────────────────────────────
JS test files:           34
JS static it() blocks:   359
JS dynamic test sources: 48
Rust integration tests:  304
Rust unit tests:         256


PER-FILE GAP ANALYSIS
────────────────────────────────────────────────────────────

[MISSING] common.js (10 static, 0 dynamic) → NO RUST EQUIVALENT
  - should expose version
  - JSON.stringify()
  - test CSS should contain all node types
  - fork()
  - generic option should work in fork()
  - custom tokenizer should be set
  - custom tokenizer should affect the parser
  - custom tokenizer should affect the lexer
  - custom tokenizer should affect the generator
  - extend nodes

[MISSING] convert.js (2 static, 0 dynamic) → NO RUST EQUIVALENT
  - fromPlainObject
  - toPlainObject

[MISSING] decode-encode.js (1 static, 8 dynamic) → NO RUST EQUIVALENT
  - (

definition-syntax-generate.js → definition_syntax_inline.rs, definition_syntax_fixtures.rs
  JS: 6 static, 1 dynamic | Rust: 60 tests
  UNMATCHED JS TESTS (6):
    - should throw an exception on bad node type (line 17)
    - ${section}/${name} (line 88)
    - prelude (line 98)
    - using forceBraces (line 117)
    - basic (line 126)
    - all the node types (line 145)
  DYNAMIC TESTS (1 sources — may generate many individual tests):
    - [dynamic] forEach-generated tests (line 60)

definition-syntax-match.js → definition_syntax_match_fixtures.rs
  JS: 6 static, 4 dynamic | Rust: 12 tests
  UNMATCHED JS TESTS (6):
    - create default syntax (line 18)
    - should MATCH to  (line 98)
    - should NOT MATCH to  (line 115)
    - match result for  (line 127)
    - should raise an error on broken type reference (line 140)
    - should raise an error on broken property reference (line 149)
  DYNAMIC TESTS (4 sources — may generate many individual tests):
    - [dynamic] forEach-generated tests (line 97)
    - [dynamic] forEach-generated tests (line 114)
    - [dynamic] forEach-generated tests (line 124)
    - [dynamic] fixture-based tests (line 138)

definition-syntax-parse.js → definition_syntax_inline.rs, definition_syntax_fixtures.rs
  JS: 3 static, 0 dynamic | Rust: 60 tests
  UNMATCHED JS TESTS (2):
    - ${section}/${name} (line 143)
    - prelude (line 153)

definition-syntax-walk.js → definition_syntax_inline.rs
  JS: 4 static, 0 dynamic | Rust: 37 tests
  UNMATCHED JS TESTS (2):
    - should throw an exception when nothing passed as walker handler (line 81)
    - should throw an exception when passed object has no enter or leave methods (line 88)

[MISSING] exports.js (10 static, 0 dynamic) → NO RUST EQUIVALENT
  - tokenizer
  - parser
  - generator
  - walker
  - convertor
  - lexer
  - definitionSyntax
  - data
  - data-patch
  - utils

find.js → walker_inline.rs
  JS: 7 static, 0 dynamic | Rust: 14 tests
  UNMATCHED JS TESTS (6):
    - using refs (line 32)
    - using context (line 40)
    - findLast (line 54)
    - using refs (line 62)
    - using context (line 70)
    - findAll (line 83)

generate.js → generator_inline.rs, generator_fixtures.rs
  JS: 5 static, 3 dynamic | Rust: 19 tests
  UNMATCHED JS TESTS (4):
    - should throws on unknown node type (line 56)
    - should generate a map (line 66)
    - should auto insert whitespaces where necessary (line 121)
    - should not insert whitespaces for values passed to tokenize() by default (line 127)
  DYNAMIC TESTS (3 sources — may generate many individual tests):
    - [dynamic] fixture-based tests (line 3)
    - [dynamic] fixture-based tests (line 54)
    - [dynamic] fixture-based tests (line 64)

lexer-check-atrule-descriptor.js → lexer_inline.rs
  JS: 4 static, 0 dynamic | Rust: 33 tests
  UNMATCHED JS TESTS (4):
    - should fail on invalid atrule (line 5)
    - should fail when at-rule has no descriptors (line 11)
    - should fail when at-rule has no descriptor (line 16)
    - should pass on correct descriptor (line 21)

lexer-check-atrule-name.js → lexer_inline.rs
  JS: 3 static, 0 dynamic | Rust: 33 tests
  UNMATCHED JS TESTS (3):
    - should pass correct atrule (line 5)
    - should pass correct vendor atrule (line 9)
    - should fail on invalid atrule (line 14)

lexer-check-atrule-prelude.js → lexer_inline.rs
  JS: 6 static, 0 dynamic | Rust: 33 tests
  UNMATCHED JS TESTS (6):
    - should fail on invalid atrule (line 5)
    - should fail when prelude is set for at-rule with no prelude (line 11)
    - should pass when no prelude for at-rule with no prelude (line 16)
    - should pass when prelude is not defined and syntax allows it (line 22)
    - should fail when prelude is not set for at-rule with prelude (line 28)
    - should pass when prelude for at-rule with prelude (line 43)

lexer-check-property-name.js → lexer_inline.rs
  JS: 3 static, 0 dynamic | Rust: 33 tests
  UNMATCHED JS TESTS (2):
    - should pass correct property (line 5)
    - should pass correct vendor property (line 9)

lexer-check-structure.js → lexer_inline.rs
  JS: 12 static, 2 dynamic | Rust: 33 tests
  UNMATCHED JS TESTS (12):
    - should fail when no structure field in node definition (line 7)
    - should fail on bad value in structure (line 17)
    - should pass correct structure (line 32)
    - should ignore properties from prototype (line 39)
    - node should be an object (line 66)
    - missed fields (line 75)
    - missed field (line 85)
    - unknown field (line 97)
    - bad data type (line 111)
    - bad loc (line 123)
    - bad loc #2 (line 135)
    - bad loc #3 (line 151)
  DYNAMIC TESTS (2 sources — may generate many individual tests):
    - [dynamic] fixture-based tests (line 3)
    - [dynamic] fixture-based tests (line 55)

lexer-match-atrule-descriptor.js → lexer_fixtures.rs
  JS: 7 static, 0 dynamic | Rust: 10 tests
  UNMATCHED JS TESTS (7):
    - should match (line 24)
    - vendor prefix in keyword name (line 32)
    - vendor prefix in declarator name (line 39)
    - case insensetive with vendor prefix (line 46)
    - should use verdor version first (line 58)
    - should not be matched to empty value (line 71)
    - should not be matched to at-rules with no descriptors (line 85)

lexer-match-atrule-prelude.js → lexer_fixtures.rs
  JS: 8 static, 0 dynamic | Rust: 10 tests
  UNMATCHED JS TESTS (8):
    - should match (line 19)
    - vendor prefix (line 27)
    - case insensetive with vendor prefix (line 34)
    - should use verdor version first (line 46)
    - should not be matched to empty value (line 59)
    - should be positive when no prelude and at-rule has no prelude (line 73)
    - regular name (line 92)
    - with verdor prefix (line 99)

lexer-match-property-iterations.js → lexer_fixtures.rs
  JS: 1 static, 0 dynamic | Rust: 10 tests
  UNMATCHED JS TESTS (1):
    - should not error on long values (line 6)

lexer-match-property.js → lexer_inline.rs, lexer_fixtures.rs
  JS: 9 static, 2 dynamic | Rust: 43 tests
  UNMATCHED JS TESTS (5):
    - hacks (line 36)
    - should use verdor version first (line 70)
    - custom property (line 83)
    - typed custom property (line 90)
    - should not be matched to empty value (line 129)
  DYNAMIC TESTS (2 sources — may generate many individual tests):
    - [dynamic] fixture-based tests (line 4)
    - [dynamic] fixture-based tests (line 143)

lexer-match-result.js → lexer_inline.rs
  JS: 4 static, 0 dynamic | Rust: 33 tests
  UNMATCHED JS TESTS (4):
    - getTrace() (line 16)
    - isType() (line 31)
    - isProperty() (line 40)
    - isKeyword() (line 49)

lexer-match-type.js → lexer_inline.rs
  JS: 4 static, 0 dynamic | Rust: 33 tests
  UNMATCHED JS TESTS (2):
    - should fail on matching wrong value (line 32)
    - should return null and save error for unknown type (line 39)

lexer-match.js → lexer_inline.rs
  JS: 5 static, 0 dynamic | Rust: 33 tests
  UNMATCHED JS TESTS (3):
    - should take a string as a value (line 24)
    - should fails on bad syntax (line 39)
    - ${syntax} -> ${value} (line 93)

lexer-relative-colors.js → lexer_fixtures.rs
  JS: 68 static, 0 dynamic | Rust: 10 tests
  UNMATCHED JS TESTS (68):
    - should match rgb(25 25 25 / 50%) (line 6)
    - should match rgb(from hsl(0 100% 50%) r g b) (line 12)
    - should match rgb(from hsl(0 100% 50%) 132 132 224) (line 18)
    - should match rgb(from #123456 calc(r + 40) calc(g + 40) b) (line 24)
    - should match rgb(from hwb(120deg 10% 20%) r g calc(b + 200)) (line 30)
    - should match rgb(25 25 25 / 50%) (line 36)
    - should match rgb(from hsl(0 100% 50%) r 80 80) (line 42)
    - should match rgb(from hsl(0 100% 50% / 0.8) r g b / alpha) (line 48)
    - should match rgb(from hsl(0 100% 50% / 0.8) r g b / 0.5) (line 54)
    - should match rgb(from hsl(0 100% 50%) calc(r/2) calc(g + 25) calc(b + 175) / calc(alpha - 0.1)) (line 60)
    - should match rgba(25 25 25) (line 68)
    - should match rgba(25 25 25 / 50%) (line 74)
    - should match rgba(from hsl(0 100% 50%) r g b) (line 80)
    - should match rgba(from hsl(0 100% 50%) 132 132 224) (line 86)
    - should match rgba(from #123456 calc(r + 40) calc(g + 40) b) (line 92)
    - should match rgba(from hwb(120deg 10% 20%) r g calc(b + 200)) (line 98)
    - should match rgba(25 25 25 / 50%) (line 104)
    - should match rgba(from hsl(0 100% 50%) r 80 80) (line 110)
    - should match rgba(from hsl(0 100% 50% / 0.8) r g b / alpha) (line 116)
    - should match rgba(from hsl(0 100% 50% / 0.8) r g b / 0.5) (line 122)
    - should match rgba(from hsl(0 100% 50%) calc(r/2) calc(g + 25) calc(b + 175) / calc(alpha - 0.1)) (line 128)
    - should match hsl(50 80% 40%) (line 136)
    - should match hsl(150deg 30% 60%) (line 142)
    - should match hsl(0.3turn 60% 45% / 0.7) (line 148)
    - should match hsl(0 80% 50% / 25%) (line 154)
    - should match hsl(none 75% 25%) (line 160)
    - should match hsl(from green h s l / 0.5) (line 166)
    - should match hsl(from #123456 h s calc(l + 20)) (line 172)
    - should match hsl(from rgb(200 0 0) calc(h + 30) s calc(l + 30)) (line 178)
    - should match hsla(50 80% 40%) (line 186)
    - should match hsla(150deg 30% 60%) (line 192)
    - should match hsla(0.3turn 60% 45% / 0.7) (line 198)
    - should match hsla(0 80% 50% / 25%) (line 204)
    - should match hsla(none 75% 25%) (line 210)
    - should match hsla(from green h s l / 0.5) (line 216)
    - should match hsla(from #123456 h s calc(l + 20)) (line 222)
    - should match hsla(from rgb(200 0 0) calc(h + 30) s calc(l + 30)) (line 228)
    - should match hwb(12 50% 0%) (line 236)
    - should match hwb(50deg 30% 40%) (line 242)
    - should match hwb(0.5turn 10% 0% / 0.5) (line 248)
    - should match hwb(0 100% 0% / 50%) (line 254)
    - should match hwb(from green h w b / 0.5) (line 260)
    - should match hwb(from #123456 h calc(w + 30) b) (line 266)
    - should match hwb(from lch(40% 70 240deg) h w calc(b - 30)) (line 272)
    - should match lab(29.2345% 39.3825 20.0664) (line 280)
    - should match lab(52.2345% 40.1645 59.9971 / .5) (line 286)
    - should match lab(from green l a b / 0.5) (line 292)
    - should match lab(from #123456 calc(l + 10) a b) (line 298)
    - should match lab(from hsl(180 100% 50%) calc(l - 10) a b) (line 304)
    - should match oklab(29.2345% 39.3825 20.0664) (line 312)
    - should match oklab(52.2345% 40.1645 59.9971 / .5) (line 318)
    - should match oklab(from green l a b / 0.5) (line 324)
    - should match oklab(from #123456 calc(l + 10) a b) (line 330)
    - should match oklab(from hsl(180 100% 50%) calc(l - 10) a b) (line 336)
    - should match lch(29.2345% 44.2 27); (line 344)
    - should match lch(52.2345% 72.2 56.2 / .5) (line 350)
    - should match lch(from green l c h / 0.5) (line 356)
    - should match lch(from #123456 calc(l + 10) c h) (line 362)
    - should match lch(from hsl(180 100% 50%) calc(l - 10) c h) (line 368)
    - should match oklch(29.2345% 44.2 27); (line 376)
    - should match oklch(52.2345% 72.2 56.2 / .5) (line 382)
    - should match oklch(from green l c h / 0.5) (line 388)
    - should match oklch(from #123456 calc(l + 10) c h) (line 394)
    - should match oklch(from hsl(180 100% 50%) calc(l - 10) c h) (line 400)
    - should match alpha(from #123456) (line 408)
    - should match alpha(from #123456 / .25) (line 414)
    - should match alpha(from hsl(0 100% 50%) / calc(0.1 * 5)) (line 420)
    - should match alpha(from rgb(25 25 25) / none) (line 426)

lexer-search-fragments.js → lexer_inline.rs
  JS: 5 static, 0 dynamic | Rust: 33 tests
  UNMATCHED JS TESTS (5):
    - should find single entry (line 14)
    - should find multiple entries (line 21)
    - should find single entry (line 30)
    - should find multiple entries (line 37)
    - should find all entries in ast (line 46)

lexer.js → lexer_inline.rs, lexer_fixtures.rs
  JS: 14 static, 0 dynamic | Rust: 43 tests
  UNMATCHED JS TESTS (13):
    - should not override generic types when used (line 5)
    - should not use generic type names when generics are not used (line 17)
    - validate() (line 28)
    - should allow override units (line 57)
    - should not add new unit groups or discard existing (line 71)
    - properties (line 111)
    - types (line 115)
    - should not append to generic (line 119)
    - default syntax shouldn\ (line 124)
    - custom syntax should not affect base syntax (line 143)
    - custom syntax should be valid and correct (line 150)
    - custom syntax should match own grammar only (line 154)
    - recovery syntax from dump (line 160)

[MISSING] list.js (60 static, 0 dynamic) → NO RUST EQUIVALENT
  - iterate
  - iterate with thisArg
  - nested iterate
  - remove items on iterate
  - insert items on iterate
  - .createItem()
  - #createItem()
  - #size
  - #fromArray()
  - #toArray()
  - #toJSON()
  - #isEmpty
  - #first
  - #last
  - #reduce()
  - #reduceRight()
  - should not iterate when start is null
  - should stop iterate when callback returns true
  - should not iterate when start is null
  - should stop iterate when callback returns true
  - basic
  - should stop on first match
  - basic
  - basic
  - empty list
  - non-empty list
  - #copy()
  - #prepend()
  - #prependData()
  - #unshift()
  - should remove first item
  - should return an undefined for an empty list
  - #append()
  - #appendData()
  - #push()
  - should remove last item
  - should return an undefined for an empty list
  - should append when no ref item
  - should insert before ref item
  - insert in the middle
  - insert the item before an item that doesn\
  - should append when no ref item
  - should insert before ref item
  - insert in the middle
  - insert the item before an item that doesn\
  - clear a list
  - clear a list in reverse order
  - remove head item that doesn\
  - remove tail item that doesn\
  - prepend non-empty list to non-empty list
  - prepend non-empty list to empty list
  - prepend empty list to non-empty
  - append non-empty list to non-empty list
  - append non-empty list to empty list
  - append empty list to non-empty
  - add non-empty list to non-empty list
  - add non-empty list to empty list
  - add empty list to non-empty
  - replace for an item
  - replace for a list

[MISSING] names.js (20 static, 9 dynamic) → NO RUST EQUIVALENT
  - base test
  - result should be immutable
  - should normalize name to lower case
  - should return the same object
  - shouldn\
  - base test
  - result should be immutable
  - should normalize name to lower case
  - name with dashes
  - should normalize vendor to lower case
  - should detect custom property
  - should detect vendor prefix and hack
  - should detect custom property and hack
  - should return the same object
  - shouldn\
  - shouldn\
  - \
  - \
  - \
  - \

nested-selector-disambiguation.js → parser_inline.rs
  JS: 7 static, 3 dynamic | Rust: 49 tests
  UNMATCHED JS TESTS (3):
    - should handle multiple nested selectors in one block (line 133)
    - should not confuse property-like selectors (line 181)
    - should parse the original failing case correctly (line 250)
  DYNAMIC TESTS (3 sources — may generate many individual tests):
    - [dynamic] forEach-generated tests (line 39)
    - [dynamic] forEach-generated tests (line 111)
    - [dynamic] forEach-generated tests (line 146)

parse-extension.js → parser_inline.rs
  JS: 6 static, 0 dynamic | Rust: 49 tests
  UNMATCHED JS TESTS (4):
    - should parse according new rules (line 37)
    - should fail on unknown (line 55)
    - should parse according new rules (line 91)
    - should fail on unknown (line 118)

parse.js → parser_inline.rs, parser_fixtures.rs
  JS: 30 static, 8 dynamic | Rust: 63 tests
  UNMATCHED JS TESTS (21):
    - should use List for children when list is true (line 156)
    - should use Array for children when list is false (line 161)
    - should call onParseError when handler is passed (line 167)
    - formattedMessage (line 200)
    - formattedMessage at eof (line 227)
    - formattedMessage (windows new lines) (line 242)
    - formattedMessage for source with long lines (line 285)
    - with no locations (line 342)
    - with locations (line 358)
    - as function (line 417)
    - should start with specified offset, line and column (line 535)
    - should parse *property as Declaration (browser hack) (line 584)
    - should parse other hack prefixes as Declaration (line 593)
    - should parse * followed by space as nested rule selector (line 607)
    - should parse * followed by space and ident as nested rule with descendant selector (line 616)
    - should throw error for *ident without whitespace in selector context (line 630)
    - should allow * followed by whitespace and ident (line 639)
    - should allow * followed by pseudo-class without whitespace (line 645)
    - should allow * followed by class without whitespace (line 650)
    - should allow * followed by attribute without whitespace (line 655)
    - should allow * followed by id without whitespace (line 660)
  DYNAMIC TESTS (8 sources — may generate many individual tests):
    - [dynamic] fixture-based tests (line 4)
    - [dynamic] fixture-based tests (line 37)
    - [dynamic] forEach-generated tests (line 49)
    - [dynamic] forEach-generated tests (line 67)
    - [dynamic] forEach-generated tests (line 79)
    - [dynamic] forEach-generated tests (line 92)
    - [dynamic] fixture-based tests (line 189)
    - [dynamic] forEach-generated tests (line 597)

tokenizer.js → tokenizer_inline.rs, tokenizer_fixtures.rs
  JS: 14 static, 3 dynamic | Rust: 18 tests
  UNMATCHED JS TESTS (5):
    - edge case: no arguments (line 58)
    - edge case: empty input (line 66)
    - should convert input to string (line 74)
    - should accept a Buffer (line 84)
    - testcase# (line 271)
  DYNAMIC TESTS (3 sources — may generate many individual tests):
    - [dynamic] forEach-generated tests (line 270)
    - [dynamic] forEach-generated tests (line 310)
    - [dynamic] fixture-based tests (line 311)

walk.js → walker_inline.rs, walker_fixtures.rs
  JS: 15 static, 5 dynamic | Rust: 34 tests
  UNMATCHED JS TESTS (7):
    - this.break (line 240)
    - this.break (line 314)
    - this.skip (line 410)
    - this.skip (line 500)
    - should throws when no enter/leave handlers is set (line 554)
    - should throws when visit has wrong value (line 566)
    - iterate DeclarationList (line 653)
  DYNAMIC TESTS (5 sources — may generate many individual tests):
    - [dynamic] fixture-based tests (line 5)
    - [dynamic] fixture-based tests (line 575)
    - [dynamic] fixture-based tests (line 581)
    - [dynamic] forEach-generated tests (line 642)
    - [dynamic] forEach-generated tests (line 643)


================================================================================
  FINAL SUMMARY
================================================================================
Total unmatched static JS tests: 312
Total JS files with no Rust equivalent: 0

MISSING TESTS BY FILE:

  [NO RUST FILE] common.js (10):
    - should expose version
    - JSON.stringify()
    - test CSS should contain all node types
    - fork()
    - generic option should work in fork()
    - custom tokenizer should be set
    - custom tokenizer should affect the parser
    - custom tokenizer should affect the lexer
    - custom tokenizer should affect the generator
    - extend nodes

  [NO RUST FILE] convert.js (2):
    - fromPlainObject
    - toPlainObject

  [NO RUST FILE] decode-encode.js (1):
    - (

  [NO RUST FILE] exports.js (10):
    - tokenizer
    - parser
    - generator
    - walker
    - convertor
    - lexer
    - definitionSyntax
    - data
    - data-patch
    - utils

  [NO RUST FILE] list.js (60):
    - iterate
    - iterate with thisArg
    - nested iterate
    - remove items on iterate
    - insert items on iterate
    - .createItem()
    - #createItem()
    - #size
    - #fromArray()
    - #toArray()
    - #toJSON()
    - #isEmpty
    - #first
    - #last
    - #reduce()
    - #reduceRight()
    - should not iterate when start is null
    - should stop iterate when callback returns true
    - should not iterate when start is null
    - should stop iterate when callback returns true
    - basic
    - should stop on first match
    - basic
    - basic
    - empty list
    - non-empty list
    - #copy()
    - #prepend()
    - #prependData()
    - #unshift()
    - should remove first item
    - should return an undefined for an empty list
    - #append()
    - #appendData()
    - #push()
    - should remove last item
    - should return an undefined for an empty list
    - should append when no ref item
    - should insert before ref item
    - insert in the middle
    - insert the item before an item that doesn\
    - should append when no ref item
    - should insert before ref item
    - insert in the middle
    - insert the item before an item that doesn\
    - clear a list
    - clear a list in reverse order
    - remove head item that doesn\
    - remove tail item that doesn\
    - prepend non-empty list to non-empty list
    - prepend non-empty list to empty list
    - prepend empty list to non-empty
    - append non-empty list to non-empty list
    - append non-empty list to empty list
    - append empty list to non-empty
    - add non-empty list to non-empty list
    - add non-empty list to empty list
    - add empty list to non-empty
    - replace for an item
    - replace for a list

  [NO RUST FILE] names.js (20):
    - base test
    - result should be immutable
    - should normalize name to lower case
    - should return the same object
    - shouldn\
    - base test
    - result should be immutable
    - should normalize name to lower case
    - name with dashes
    - should normalize vendor to lower case
    - should detect custom property
    - should detect vendor prefix and hack
    - should detect custom property and hack
    - should return the same object
    - shouldn\
    - shouldn\
    - \
    - \
    - \
    - \

  definition-syntax-generate.js (6):
    - should throw an exception on bad node type
    - ${section}/${name}
    - prelude
    - using forceBraces
    - basic
    - all the node types

  definition-syntax-match.js (6):
    - create default syntax
    - should MATCH to 
    - should NOT MATCH to 
    - match result for 
    - should raise an error on broken type reference
    - should raise an error on broken property reference

  definition-syntax-parse.js (2):
    - ${section}/${name}
    - prelude

  definition-syntax-walk.js (2):
    - should throw an exception when nothing passed as walker handler
    - should throw an exception when passed object has no enter or leave methods

  find.js (6):
    - using refs
    - using context
    - findLast
    - using refs
    - using context
    - findAll

  generate.js (4):
    - should throws on unknown node type
    - should generate a map
    - should auto insert whitespaces where necessary
    - should not insert whitespaces for values passed to tokenize() by default

  lexer-check-atrule-descriptor.js (4):
    - should fail on invalid atrule
    - should fail when at-rule has no descriptors
    - should fail when at-rule has no descriptor
    - should pass on correct descriptor

  lexer-check-atrule-name.js (3):
    - should pass correct atrule
    - should pass correct vendor atrule
    - should fail on invalid atrule

  lexer-check-atrule-prelude.js (6):
    - should fail on invalid atrule
    - should fail when prelude is set for at-rule with no prelude
    - should pass when no prelude for at-rule with no prelude
    - should pass when prelude is not defined and syntax allows it
    - should fail when prelude is not set for at-rule with prelude
    - should pass when prelude for at-rule with prelude

  lexer-check-property-name.js (2):
    - should pass correct property
    - should pass correct vendor property

  lexer-check-structure.js (12):
    - should fail when no structure field in node definition
    - should fail on bad value in structure
    - should pass correct structure
    - should ignore properties from prototype
    - node should be an object
    - missed fields
    - missed field
    - unknown field
    - bad data type
    - bad loc
    - bad loc #2
    - bad loc #3

  lexer-match-atrule-descriptor.js (7):
    - should match
    - vendor prefix in keyword name
    - vendor prefix in declarator name
    - case insensetive with vendor prefix
    - should use verdor version first
    - should not be matched to empty value
    - should not be matched to at-rules with no descriptors

  lexer-match-atrule-prelude.js (8):
    - should match
    - vendor prefix
    - case insensetive with vendor prefix
    - should use verdor version first
    - should not be matched to empty value
    - should be positive when no prelude and at-rule has no prelude
    - regular name
    - with verdor prefix

  lexer-match-property-iterations.js (1):
    - should not error on long values

  lexer-match-property.js (5):
    - hacks
    - should use verdor version first
    - custom property
    - typed custom property
    - should not be matched to empty value

  lexer-match-result.js (4):
    - getTrace()
    - isType()
    - isProperty()
    - isKeyword()

  lexer-match-type.js (2):
    - should fail on matching wrong value
    - should return null and save error for unknown type

  lexer-match.js (3):
    - should take a string as a value
    - should fails on bad syntax
    - ${syntax} -> ${value}

  lexer-relative-colors.js (68):
    - should match rgb(25 25 25 / 50%)
    - should match rgb(from hsl(0 100% 50%) r g b)
    - should match rgb(from hsl(0 100% 50%) 132 132 224)
    - should match rgb(from #123456 calc(r + 40) calc(g + 40) b)
    - should match rgb(from hwb(120deg 10% 20%) r g calc(b + 200))
    - should match rgb(25 25 25 / 50%)
    - should match rgb(from hsl(0 100% 50%) r 80 80)
    - should match rgb(from hsl(0 100% 50% / 0.8) r g b / alpha)
    - should match rgb(from hsl(0 100% 50% / 0.8) r g b / 0.5)
    - should match rgb(from hsl(0 100% 50%) calc(r/2) calc(g + 25) calc(b + 175) / calc(alpha - 0.1))
    - should match rgba(25 25 25)
    - should match rgba(25 25 25 / 50%)
    - should match rgba(from hsl(0 100% 50%) r g b)
    - should match rgba(from hsl(0 100% 50%) 132 132 224)
    - should match rgba(from #123456 calc(r + 40) calc(g + 40) b)
    - should match rgba(from hwb(120deg 10% 20%) r g calc(b + 200))
    - should match rgba(25 25 25 / 50%)
    - should match rgba(from hsl(0 100% 50%) r 80 80)
    - should match rgba(from hsl(0 100% 50% / 0.8) r g b / alpha)
    - should match rgba(from hsl(0 100% 50% / 0.8) r g b / 0.5)
    - should match rgba(from hsl(0 100% 50%) calc(r/2) calc(g + 25) calc(b + 175) / calc(alpha - 0.1))
    - should match hsl(50 80% 40%)
    - should match hsl(150deg 30% 60%)
    - should match hsl(0.3turn 60% 45% / 0.7)
    - should match hsl(0 80% 50% / 25%)
    - should match hsl(none 75% 25%)
    - should match hsl(from green h s l / 0.5)
    - should match hsl(from #123456 h s calc(l + 20))
    - should match hsl(from rgb(200 0 0) calc(h + 30) s calc(l + 30))
    - should match hsla(50 80% 40%)
    - should match hsla(150deg 30% 60%)
    - should match hsla(0.3turn 60% 45% / 0.7)
    - should match hsla(0 80% 50% / 25%)
    - should match hsla(none 75% 25%)
    - should match hsla(from green h s l / 0.5)
    - should match hsla(from #123456 h s calc(l + 20))
    - should match hsla(from rgb(200 0 0) calc(h + 30) s calc(l + 30))
    - should match hwb(12 50% 0%)
    - should match hwb(50deg 30% 40%)
    - should match hwb(0.5turn 10% 0% / 0.5)
    - should match hwb(0 100% 0% / 50%)
    - should match hwb(from green h w b / 0.5)
    - should match hwb(from #123456 h calc(w + 30) b)
    - should match hwb(from lch(40% 70 240deg) h w calc(b - 30))
    - should match lab(29.2345% 39.3825 20.0664)
    - should match lab(52.2345% 40.1645 59.9971 / .5)
    - should match lab(from green l a b / 0.5)
    - should match lab(from #123456 calc(l + 10) a b)
    - should match lab(from hsl(180 100% 50%) calc(l - 10) a b)
    - should match oklab(29.2345% 39.3825 20.0664)
    - should match oklab(52.2345% 40.1645 59.9971 / .5)
    - should match oklab(from green l a b / 0.5)
    - should match oklab(from #123456 calc(l + 10) a b)
    - should match oklab(from hsl(180 100% 50%) calc(l - 10) a b)
    - should match lch(29.2345% 44.2 27);
    - should match lch(52.2345% 72.2 56.2 / .5)
    - should match lch(from green l c h / 0.5)
    - should match lch(from #123456 calc(l + 10) c h)
    - should match lch(from hsl(180 100% 50%) calc(l - 10) c h)
    - should match oklch(29.2345% 44.2 27);
    - should match oklch(52.2345% 72.2 56.2 / .5)
    - should match oklch(from green l c h / 0.5)
    - should match oklch(from #123456 calc(l + 10) c h)
    - should match oklch(from hsl(180 100% 50%) calc(l - 10) c h)
    - should match alpha(from #123456)
    - should match alpha(from #123456 / .25)
    - should match alpha(from hsl(0 100% 50%) / calc(0.1 * 5))
    - should match alpha(from rgb(25 25 25) / none)

  lexer-search-fragments.js (5):
    - should find single entry
    - should find multiple entries
    - should find single entry
    - should find multiple entries
    - should find all entries in ast

  lexer.js (13):
    - should not override generic types when used
    - should not use generic type names when generics are not used
    - validate()
    - should allow override units
    - should not add new unit groups or discard existing
    - properties
    - types
    - should not append to generic
    - default syntax shouldn\
    - custom syntax should not affect base syntax
    - custom syntax should be valid and correct
    - custom syntax should match own grammar only
    - recovery syntax from dump

  nested-selector-disambiguation.js (3):
    - should handle multiple nested selectors in one block
    - should not confuse property-like selectors
    - should parse the original failing case correctly

  parse-extension.js (4):
    - should parse according new rules
    - should fail on unknown
    - should parse according new rules
    - should fail on unknown

  parse.js (21):
    - should use List for children when list is true
    - should use Array for children when list is false
    - should call onParseError when handler is passed
    - formattedMessage
    - formattedMessage at eof
    - formattedMessage (windows new lines)
    - formattedMessage for source with long lines
    - with no locations
    - with locations
    - as function
    - should start with specified offset, line and column
    - should parse *property as Declaration (browser hack)
    - should parse other hack prefixes as Declaration
    - should parse * followed by space as nested rule selector
    - should parse * followed by space and ident as nested rule with descendant selector
    - should throw error for *ident without whitespace in selector context
    - should allow * followed by whitespace and ident
    - should allow * followed by pseudo-class without whitespace
    - should allow * followed by class without whitespace
    - should allow * followed by attribute without whitespace
    - should allow * followed by id without whitespace

  tokenizer.js (5):
    - edge case: no arguments
    - edge case: empty input
    - should convert input to string
    - should accept a Buffer
    - testcase#

  walk.js (7):
    - this.break
    - this.break
    - this.skip
    - this.skip
    - should throws when no enter/leave handlers is set
    - should throws when visit has wrong value
    - iterate DeclarationList
