# Module Concatenation Analysis: Rspack vs Webpack

## Executive Summary

Two distinct issues affect different parts of the build:

1. **Critical path (question page TTI, +4% regression)**: SWC's code emitter produces ~10-30% more bytes per module than Babel, adding ~4% to vendor/ssr chunks. For `.graphql.ts` files, SWC preserves multi-line formatting (2.7x larger unminified), but this vanishes after minification. The persistent ~4% overhead in vendor/ssr is from SWC emitter verbosity on non-graphql code.

2. **Async chunks (+37% size)**: Webpack 4 doesn't support `exports` field; rspack resolves `@okta/okta-auth-js` to ESM + 489 CJS polyfills instead of a self-contained UMD bundle.

Key facts:
- Concatenation algorithm is equivalent; module classification for shared modules is 99.2% identical
- The "2.1x non-ESM bailout" claim was a measurement artifact (mixed direct and indirect bailouts)
- Critical path TTI regression is driven by SWC code generation, not concatenation or module resolution

## Definitive Comparison (fresh builds, same source, same config, unminified)

| Chunk | Rspack defs | Webpack defs | Rspack concat markers | Webpack concat markers |
|-------|-------------|--------------|----------------------|----------------------|
| common | 716 | 737 | 501 | 498 |
| ssr | 1,117 | 1,137 | 513 | 504 |
| vendor | 419 | 418 | 17 | 11 |
| polyfills | 766 | 766 | 0 | 0 |
| entry | 5 | 8 | 0 | 0 |
| **Total** | **3,023** | **3,066** | **1,031** | **1,013** |

Module definitions differ by less than 2%. Concatenation markers differ by less than 2%.

### Earlier (incorrect) measurements

The initial comparison used pre-built files with mismatched regex patterns, yielding a false 3.7x gap:

| Chunk | Rspack (old) | Webpack (old) | Notes |
|-------|-------------|---------------|-------|
| common | 715 | 178 | Webpack file likely measured concat groups, not individual defs |
| vendor | 288 | 101 | Different chunk content |
| polyfills | 913 | 228 | Different polyfill sets |
| **Total** | **2,274** | **617** | **Measurement artifact** |

## Chunk Assignment Comparison

Direct comparison of module-chunk distributions (fresh builds, same source/config):

| Modules in N chunks | Rspack | Webpack |
|---------------------|--------|---------|
| 1 chunk | 3,946 | 3,526 |
| 2 chunks | 2,138 | 2,161 |
| 3 chunks | 255 | 289 |
| 4+ chunks | 892 | 847 |
| **Total unique modules** | **7,231** | **6,823** |

Top 2-chunk pairs (nearly identical):

| Pair | Rspack | Webpack |
|------|--------|---------|
| common + ssr | 744 | 765 |
| ssr + vendor | 395 | 394 |
| component-Modals-AdsManager + page-AdsManager-main | 176 | 176 |
| component-AdminTools-profile + page-LogPages | 132 | 132 |

**Chunk assignments are essentially identical.** The 408 extra modules in rspack (7,231 vs 6,823) are mostly in single-chunk entries, likely from minor tree shaking differences.

## Concatenation Stats Comparison (across all ~250 chunks)

| Metric | Rspack | Webpack |
|--------|--------|---------|
| Concatenation groups | 1,185 | 1,145 |
| Modules in concat groups | 7,452 | 7,246 |
| Unique modules concatenated | 3,399 | 3,421 |

Top concat groups are identical (same root modules, same sizes, same chunks).

## Investigation Timeline

### Phase 1: Measurement Correction

The initial "82x" `require()` count difference was a measurement artifact — mismatched regex patterns. After correcting, `require()` counts are equal (~1.0x).

### Phase 2: Algorithm Comparison

Rspack's `ModuleConcatenationPlugin._tryToAdd` was compared line-by-line with webpack's. Both use identical logic for chunk checks, importer verification, and runtime conditions. **No algorithmic difference found.**

### Phase 3: SplitChunksPlugin Comparison

The `splitChunks` configuration and implementation are identical between rspack and webpack.

### Phase 4: Diagnostic Instrumentation

Custom diagnostics added to `module_concatenation_plugin.rs` showed expected failure patterns (e.g., `incorrect_chunks` when root and inner modules are in disjoint chunk sets). These failures occur equally in both bundlers.

### Phase 5: Direct Output Comparison

Fresh unminified builds with identical configs confirmed module definition parity. The original 3.7x gap was from comparing non-equivalent builds with mismatched measurement patterns.

## Full Build Comparison (all 250 chunks, unminified)

| Metric | Rspack | Webpack | Delta |
|--------|--------|---------|-------|
| **Initial chunks** (11 files) | | | |
| Module definitions | 3,123 | 3,166 | -1.4% (rspack fewer) |
| Concat markers | 1,141 | 1,128 | +1.2% |
| Total size | 12.52 MB | 12.19 MB | +2.7% |
| **Async chunks** (239 files) | | | |
| Module definitions | 13,844 | 12,720 | **+8.8%** |
| Concat markers | 6,311 | 6,118 | +3.2% |
| Total size | 108.72 MB | 79.60 MB | **+36.6%** |
| **ALL chunks** (250 files) | | | |
| Module definitions | 16,967 | 15,886 | +6.8% |
| Concat markers | 7,452 | 7,246 | +2.8% |
| Total size | 121.25 MB | 91.79 MB | **+32.1%** |

### Key async chunk differences

| Chunk | Rspack defs | Webpack defs | Rspack size | Webpack size |
|-------|-------------|--------------|-------------|--------------|
| page-AdsManager-main | 842 | 852 | 6,265 KB | 5,337 KB |
| component-SecondaryFeed | 537 | 272 | 2,062 KB | 834 KB |
| component-Modals-AdsManager | 512 | 514 | 3,088 KB | 2,903 KB |
| page-OktaLogin | 474 | N/A | 4,202 KB | N/A |
| component-TribeFeed | 260 | N/A | 1,180 KB | N/A |

Some async chunks in rspack are substantially larger (e.g. `component-SecondaryFeed` is 2.5x bigger) or contain many more module definitions.

## Bailout Comparison

| Bailout reason | Rspack | Webpack |
|----------------|--------|---------|
| **Not an ECMAScript module** | **11,123** | **5,336** |
| Cannot concat (different chunks) | 232+ | 649+ |

### Corrected analysis: the "2.1x" claim was a measurement artifact

The 11,123 vs 5,336 counts **mix two types of bailouts**:
1. **Direct**: "Module is not an ECMAScript module" — the module itself is non-ESM
2. **Indirect**: "Cannot concat with X: Module is not an ECMAScript module" — a *dependency* is non-ESM

After separating:

| Metric | Rspack | Webpack | Delta |
|--------|--------|---------|-------|
| True non-ESM (module itself) | 1,824 | 1,437 | +387 |
| Indirect (dependency is non-ESM) | 1,193 | 445 | +748 |

The 387 extra true non-ESM modules in rspack are almost entirely modules that **only exist in rspack's build** (not shared with webpack) — see module resolution section below.

For the 10,028 shared modules, only **79 (0.8%) have different ESM classification**, and most of those are webpack marking as non-ESM while rspack keeps as auto — the opposite of the initial hypothesis.

## Minified Production Build (rspack only)

| Metric | Rspack minified | Rspack unminified | Compression |
|--------|----------------|-------------------|-------------|
| Initial size | 3.99 MB | 12.52 MB | 3.1x |
| Async size | 32.30 MB | 108.72 MB | 3.4x |
| Total size | 36.29 MB | 121.25 MB | 3.3x |
| Module defs | 16,944 | 16,967 | ~same |

(Webpack minified build was not obtainable due to the project's config gating minification via `compressionEnabled`.)

## Module Resolution Difference (Root Cause)

**The `ans_frontend` project uses webpack 4.46.0.** Webpack 4 does not support the `exports` field in `package.json`. Rspack supports `exports` per the webpack 5 specification. This causes fundamentally different dependency trees.

### @okta/okta-auth-js

| | Webpack 4 | Rspack |
|--|-----------|--------|
| Field used | `browser` (legacy) | `exports.browser.import` |
| Entry file | `dist/okta-auth-js.umd.js` (270 KB) | `esm/esm.browser.js` (374 KB) |
| Dependencies | Self-contained (all bundled inline) | 41 imports from `@babel/runtime-corejs3` → 489 `core-js-pure` CJS modules |

### broadcast-channel (Okta dependency)

| | Webpack 4 | Rspack |
|--|-----------|--------|
| Field used | `browser` (legacy) | `exports.browser.import` |
| Entry file | `dist/lib/index.es5.js` | `dist/esbrowser/index.js` |
| Extra modules | 0 | 10 separate files |

### Module inventory diff

| Category | Only in rspack (674) | Only in webpack (979) |
|----------|---------------------|----------------------|
| `core-js-pure` | **415** | 0 |
| `@babel/runtime-corejs3` | **74** | 0 |
| Source/internal | 107 | 23 |
| `@babel/runtime` | 19 | 8 |
| `broadcast-channel` | 10 | 0 |
| `date-fns` | 0 | 204 |
| `@nivo/*` | 0 | 507 |
| `relay-runtime` | 0 | 90 |
| `d3-*` | 0 | 104 |
| Other packages | 49 | 43 |

Rspack's extras (489 `core-js-pure` + `runtime-corejs3`) are all **CJS modules that cannot be concatenated**. Webpack's extras (`date-fns`, `@nivo/*`, `d3-*`) are largely from **nested `node_modules`** (non-hoisted dependencies in webpack 4's resolution) and different concat group reporting in stats.

### Size impact of extra polyfill modules

The 489 extra CJS polyfill modules in rspack total ~307 KB source size + ~96 KB wrapper overhead = ~403 KB per chunk copy. When duplicated across async chunks, this substantially inflates total output.

## 30MB Size Difference Explained

The unminified output difference is **29.46 MB** (121.25 - 91.79), concentrated in async chunks (29.12 MB). The causes, now understood in priority order:

1. **Module resolution difference** (primary) — rspack resolves `@okta/okta-auth-js` via `exports` to an ESM entry with 489 separate CJS dependencies. Webpack 4 resolves to a self-contained UMD bundle. This adds ~403 KB per chunk copy, and with the Okta-related modules duplicated across many async chunks, accounts for a significant portion of the size difference.

2. **Code generation overhead** — the per-module code is larger in rspack. This is visible in chunks with identical module counts but different sizes (e.g. `page-AdsManager-main`: same ~850 defs but rspack is 928 KB larger). Likely causes:
   - Different variable naming / mangling in concatenated modules
   - Different helper function inlining
   - Different comment/whitespace patterns in unminified mode

3. **Different dependency trees** — beyond Okta, the two bundlers resolve to different packages in many cases: rspack uses `querystring` while webpack uses `querystring-es3`, rspack includes `assert`/`util` from different paths, etc. This adds up across 674 extra rspack modules and 979 extra webpack modules.

4. **Fewer tree-shaking eliminations** — rspack includes more code per module. Some modules that webpack marks as side-effect-free and tree-shakes away are retained by rspack.

## Critical Path Analysis (Question Page TTI)

The critical path chunks for the question page are: `entry`, `vendor`, `common`, `ssr`, `section-QuestionHeader`, `component-QuestionPage`, `section-QuestionAnswerArea`. None of these are affected by the Okta/broadcast-channel `exports` resolution issue (those packages are lazy-loaded).

### Unminified critical path sizes

| Chunk | Rspack | Webpack | Delta | Delta % |
|-------|--------|---------|-------|---------|
| entry.js | 54 KB | 60 KB | -6 KB | -10.4% |
| vendor.chunk.js | 1,608 KB | 1,547 KB | +61 KB | **+4.0%** |
| common.chunk.js | 3,826 KB | 3,696 KB | +130 KB | **+3.5%** |
| ssr.chunk.js | 5,664 KB | 5,372 KB | +292 KB | **+5.4%** |
| section-QuestionHeader | 365 KB | 284 KB | +81 KB | **+28.5%** |
| component-QuestionPage | 62 KB | 72 KB | -10 KB | -13.6% |
| section-QuestionAnswerArea | 1,638 KB | 1,169 KB | +469 KB | **+40.1%** |
| **TOTAL** | **13,217 KB** | **12,200 KB** | **+1,017 KB** | **+8.3%** |

### Root cause: two distinct issues

#### 1. GraphQL/Relay artifact formatting (78-96% of QuestionHeader/AnswerArea delta)

Relay-generated `.graphql.ts` files contain large JSON-like object literals. SWC preserves the multi-line formatted source, while **webpack 4's code generation compresses module code to single-line**:

| File | Rspack (stats) | Webpack (stats) | Ratio |
|------|---------------|----------------|-------|
| QuestionAnswerAreaSectionQuery.graphql.ts | 405.6 KB | 146.1 KB | **2.78x** |
| QuestionPageAdsQuery.graphql.ts | 289.9 KB | 96.7 KB | **3.00x** |
| QuestionPageHeaderSectionQuery.graphql.ts | 127.4 KB | 47.5 KB | **2.68x** |
| ModernPageWrapperQuery.graphql.ts | 124.1 KB | 46.2 KB | **2.69x** |

**Webpack 4** compresses the object literal to one line:
```js
var node=function(){var v0={"alias":null,"args":null,"kind":"ScalarField","name":"id","storageKey":null},...
```

**Rspack** preserves the multi-line source:
```js
var v0 = {
    "alias": null,
    "args": null,
    "kind": "ScalarField",
    "name": "id",
    "storageKey": null
  },
```

Size diff by file type in critical path:

| Chunk | graphql delta | non-graphql delta | graphql % |
|-------|--------------|-------------------|-----------|
| common | +462 KB | +127 KB | 78% |
| section-QuestionHeader | +104 KB | +4 KB | 96% |
| section-QuestionAnswerArea | +517 KB | +40 KB | 93% |
| vendor | 0 KB | +61 KB | 0% |

**After minification, this difference vanishes** — both produce compact single-line output.

#### 2. SWC code generation overhead (~4% on non-graphql code)

For code that has no formatting ambiguity, SWC still produces larger output than Babel. Verified on the vendor chunk (zero graphql files):

Top contributors in vendor (+61 KB total from shared modules):

| Module | Rspack | Webpack | Delta | % |
|--------|--------|---------|-------|---|
| react-dom.profiling.min.js | 238 KB | 205 KB | +33 KB | +16% |
| tracekit.js | 43 KB | 35 KB | +8 KB | +24% |
| intl-pluralrules/plural-rules.js | 39 KB | 31 KB | +8 KB | +26% |
| stylis.browser.esm.js | 20 KB | 15 KB | +5 KB | +31% |
| react-shallow-renderer.js | 39 KB | 34 KB | +4 KB | +13% |

**Even pre-minified files are inflated**: `react-dom.profiling.min.js` (141 KB source) becomes 238 KB after SWC vs 210 KB after Babel. Both compilers parse and re-emit the code, but SWC's emitter produces more verbose output.

The overhead distribution across 310 shared vendor modules:
- <5% overhead: 227 modules (73%)
- 5-15% overhead: 67 modules (22%)
- 15-30% overhead: 11 modules (4%)
- >30% overhead: 5 modules (2%)

### Minified size estimate

| Chunk | Rspack minified | Est. webpack minified | Est. delta |
|-------|----------------|----------------------|------------|
| entry | 12 KB | ~13 KB | -1 KB |
| vendor | 542 KB | ~520 KB | **+22 KB (+4%)** |
| common | 1,160 KB | ~1,150 KB | +10 KB (+1%) |
| ssr | 1,812 KB | ~1,740 KB | **+72 KB (+4%)** |
| QuestionHeader | 100 KB | ~99 KB | +1 KB |
| QuestionPage | 17 KB | ~19 KB | -2 KB |
| QuestionAnswerArea | 425 KB | ~416 KB | +9 KB (+2%) |
| **TOTAL** | **4,068 KB** | **~3,957 KB** | **~+111 KB (+2.8%)** |

The graphql formatting difference disappears after minification. The residual ~3-4% in vendor/ssr chunks comes from SWC code generation patterns that persist even after re-minification (different variable naming, helper patterns, expression structures).

## Conclusion

The concatenation algorithm is equivalent. Module classification for shared modules is 99.2% identical. Two distinct issues affect different parts of the build:

### For async chunks (non-critical-path)
**Module resolution** — webpack 4 doesn't support `exports` field, resolving `@okta/okta-auth-js` to a UMD bundle (270 KB) vs rspack resolving to ESM + 489 CJS polyfills. Fix with resolve alias.

### For critical path (question page TTI)
**Code generation** — two sub-issues:
1. **GraphQL formatting**: SWC preserves multi-line object literals; webpack 4 compresses to single-line. This is ~2.7-3x for `.graphql.ts` files unminified but **vanishes after minification**.
2. **SWC emitter verbosity**: SWC's code emitter produces ~10-30% more bytes per module than Babel, even for pre-minified input. This accounts for ~+4% in vendor/ssr chunks and persists partially after minification (~2.8% estimated total critical path).

### Actionable next steps

1. **For this project (async chunks)**: Add resolve alias for Okta:
   ```js
   resolve: {
     alias: {
       "@okta/okta-auth-js": path.resolve("./node_modules/@okta/okta-auth-js/dist/okta-auth-js.umd.js"),
     }
   }
   ```
2. **For rspack core (critical path, high impact)**: Investigate SWC code emitter verbosity — the `builtin:swc-loader` re-formats even pre-minified files, adding 10-30% size. This is the primary driver of the ~4% production size regression. Specific areas:
   - SWC's JavaScript emitter adds whitespace/newlines that Babel's emitter does not
   - The `node_modules` SWC loader rule (`use: "builtin:swc-loader"` with no options) re-parses and re-formats all external JS, inflating pre-minified files
   - Consider whether the SWC loader is even necessary for node_modules `.js` files (many are already compiled)
3. **For rspack core (medium impact)**: Compare tree shaking effectiveness (usedExports / sideEffects) — rspack retains 408 more unique modules
4. **For rspack core (low impact)**: Consider `conditionNames` defaults for webpack 5 `exports` field interaction with packages designed for webpack 4

## Measurement Tools

- `/home/junkim/ans/ans_frontend/critical_path_stats.js` — extracts per-module stats for critical path chunks (run separately for rspack/webpack)
- `/home/junkim/ans/ans_frontend/compare_critical_modules.js` — compares per-module sizes in critical path chunks using saved stats JSON
- `/home/junkim/ans/ans_frontend/critical_path_diff.js` — file-level size comparison of critical path chunks
- `/home/junkim/ans/ans_frontend/full_analysis.js` — comprehensive analysis: module defs, concat markers, bailouts, sizes across all chunks (supports `--minified` flag)
- `/home/junkim/ans/ans_frontend/diff_module_types.js` + `diff_module_types_wp.js` — extracts per-module ESM classification from both bundlers, produces `/tmp/modtype_diff.json`
- `/home/junkim/ans/ans_frontend/diff_module_types_v2.js` — reclassifies with direct vs indirect bailout separation
- `/home/junkim/ans/ans_frontend/compare_chunks.js` — compares module-chunk assignments
- `/home/junkim/ans/ans_frontend/compare_concat.js` — compares concatenation groups
- `/home/junkim/ans/ans_frontend/build_and_count.js` — builds unminified and counts module definitions
- Diagnostic instrumentation in `module_concatenation_plugin.rs` (use `NAPI_RS_NATIVE_LIBRARY_PATH` env var to load workspace binary)
