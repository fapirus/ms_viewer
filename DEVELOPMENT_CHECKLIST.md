# Development Checklist

이 문서는 실제 개발 진행표다.
각 항목은 구현 완료와 테스트 검증이 끝났을 때만 체크한다.

## Working rules
- 큰 기능은 `Rust 엔진 구현 -> 테스트 -> Flutter 연결 -> 테스트` 순서로 진행한다.
- 각 단계는 fixture 또는 자동 테스트가 있어야 완료로 본다.
- 체크는 코드와 테스트가 모두 들어간 뒤에만 한다.
- 새 범위가 생기면 이 문서에 먼저 체크박스로 추가한 뒤 작업한다.

## Definition of done
하나의 체크박스를 완료로 표시하려면 아래 조건을 만족해야 한다.
- 구현 코드가 들어갔다.
- 영향 범위에 맞는 Rust 테스트가 추가되었다.
- 영향 범위에 맞는 Flutter 테스트 또는 샘플 검증이 추가되었다.
- 관련 fixture가 필요하면 추가되었다.
- 문서가 필요한 경우 `docs/architecture/` 또는 `PROJECT_PLAN.md`에 반영되었다.

## Global foundation
- [x] FFI contract 초안 확정
  - Deliverable: `docs/architecture/ffi_contract.md`
  - Rust test: request/response type serialization test
  - Flutter test: contract model decode smoke test
  - Done: initial open document request/success/error contract added
- [x] Render model 초안 확정
  - Deliverable: `docs/architecture/render_model.md`
  - Rust test: page/render node shape test
  - Flutter test: render model consume smoke test
  - Done: initial page/text/image/box/selection anchor model added
- [x] Font fallback policy 초안 확정
  - Deliverable: `docs/architecture/font_fallback_policy.md`
  - Rust test: fallback resolution table test
  - Flutter test: font selection policy smoke test
  - Done: initial platform font mapping and script fallback table added
- [x] Cache policy 초안 확정
  - Deliverable: `docs/architecture/cache_policy.md`
  - Rust test: cache eviction unit test
  - Flutter test: viewport page request smoke test
  - Done: initial LRU cache policy and viewport request model added
- [x] Crypto flow 초안 확정
  - Deliverable: `docs/architecture/crypto_flow.md`
  - Rust test: wrong password / unsupported encryption error mapping test
  - Flutter test: password prompt flow state test
  - Done: initial crypto error mapping and password prompt state model added
- [x] Fixture 운영 규칙 문서화
  - Deliverable: `fixtures/README.md`
  - Rust test: fixture discovery smoke test
  - Flutter test: none required
  - Done: fixture directory convention and discovery smoke test added

## Phase 0: Engine foundation
### Archive and package layer
- [x] OOXML archive reader 구현
  - Rust scope: ZIP central directory read, part lookup, lazy entry open
  - Tests:
    - valid zip open
    - missing part lookup
    - invalid zip failure
    - large entry lazy read behavior
  - Done: path-based OOXML archive open, part lookup, on-demand part read implemented
- [x] input abstraction 구현 (`PathSource`, `BytesSource`)
  - Rust scope: common input interface
  - Tests:
    - same document opens from path and bytes
    - invalid source error mapping
  - Done: archive source abstraction added for path and in-memory bytes
- [x] encrypted package 감지 구현
  - Rust scope: encrypted package detection without full parse
  - Tests:
    - plain package fixture
    - encrypted package fixture
    - unsupported encryption fixture
  - Done: package kind detector added for zip, encrypted OLE, and unsupported OLE containers

### XML and shared model
- [x] XML parsing 유틸 구현
  - Rust scope: namespace aware element traversal, attribute helpers
  - Tests:
    - namespace handling
    - missing attribute handling
    - malformed xml failure
  - Done: lightweight XML element parser and attribute helpers added
- [x] shared OOXML relationship parser 구현
  - Rust scope: `_rels`, content types, part resolution
  - Tests:
    - package relationships fixture
    - missing relationship target
  - Done: content types, package relationships, and target resolution parser added
- [x] shared text/run/style base model 정의
  - Rust scope: common text span, block, image reference model
  - Tests:
    - model construction tests
    - serde round-trip tests
  - Done: parser-facing shared text run, block, and image reference model added

### FFI and Flutter foundation
- [x] Rust error model과 Flutter error mapping 연결
  - Done: snake_case wire error decoding, Flutter exception mapping, password prompt reducer
  - Tests:
    - Rust error conversion test
    - Flutter error presentation test
- [x] document open FFI skeleton 구현
  - Done: openDocument request/result contract, Rust package open skeleton, default platform not-implemented path
  - Tests:
    - open success smoke test
    - password required state smoke test
- [x] Flutter viewer shell 구현
  - Done: controller open flow, password replay shell, loading/error/password/document placeholder states
  - Flutter scope: placeholder viewport, page switch shell, loading state
  - Tests:
    - widget smoke test
    - loading/empty/error state widget tests

## Phase 1: DOCX MVP
### DOCX parse layer
- [x] DOCX package entry points 구현
  - Done: main document, styles, numbering, header/footer, media relationship entry points
  - Rust scope: main document, styles, numbering, media, header/footer lookup
  - Tests:
    - minimal docx fixture
    - missing optional parts fixture
- [x] paragraph and run parser 구현
  - Done: paragraph block parsing, direct run formatting, inline line break handling
  - Rust scope: paragraphs, runs, line breaks, basic inline formatting
  - Tests:
    - styled paragraph fixture
    - mixed formatting fixture
- [x] style resolution 구현
  - Done: docDefaults, character style basedOn chain, direct formatting override merge
  - Rust scope: default style, named style, direct formatting merge
  - Tests:
    - base style inheritance fixture
    - direct formatting override fixture
- [x] numbering/list parser 구현
  - Done: numbering.xml abstractNum/num mapping, paragraph list marker extraction, nested list levels
  - Rust scope: bullet and decimal list model
  - Tests:
    - nested list fixture
    - numbering override fixture
- [x] table parser 구현
  - Done: table row/cell model, cell paragraph parsing, gridSpan/vMerge fallback metadata
  - Rust scope: rows, cells, cell text
  - Tests:
    - basic table fixture
    - merged cell fallback handling fixture
- [x] image reference parser 구현
  - Done: drawing blip relationship resolution, image block emission, missing media relationship validation
  - Rust scope: drawing/image relationship resolution
  - Tests:
    - inline image fixture
    - missing media relationship fixture
- [x] header/footer parser 구현
  - Done: sectPr section parsing, header/footer relationship resolution, section break handling
  - Rust scope: section linked headers and footers
  - Tests:
    - different first page fixture
    - section break fixture

### DOCX layout layer
- [x] page box model 구현
  - Done: sectPr page size/margin parsing and content frame calculation
  - Rust scope: page size, margins, content frame
  - Tests:
    - page metrics fixture
- [x] block layout MVP 구현
  - Done: simple paragraph flow, estimated line breaking, explicit page break handling
  - Rust scope: paragraph flow, line breaking, page breaking
  - Tests:
    - multi page paragraph fixture
    - explicit page break fixture
- [x] header/footer layout 구현
  - Done: header/footer margin region placement and section-aware overlay positioning
  - Tests:
    - header/footer position fixture
- [x] image and table block placement 구현
  - Done: image flow placement, table row/cell layout metadata, row-aware page splitting
  - Tests:
    - image flow fixture
    - table page split behavior fixture

### DOCX interaction layer
- [x] DOCX text extraction and search index 구현
  - Done: page text extraction, case-insensitive search matches, Flutter search result state and list shell
  - Rust tests:
    - simple query match fixture
    - case-insensitive match fixture
  - Flutter tests:
    - search result list widget test
    - next/previous navigation state test
- [x] DOCX text selection metadata 구현
  - Done: page text nodes, per-line bounds, selection anchors, Flutter highlight and drag state shell
  - Rust tests:
    - text box bounds generation test
    - cross-line selection anchors test
  - Flutter tests:
    - selection highlight widget test
    - drag selection state test

### DOCX integration
- [x] DOCX render model to Flutter painting 연결
  - Done: page render widget, custom painter, MsDocumentView preview page consumption
  - Flutter tests:
    - page widget smoke test
    - placeholder paint test from mock render model
- [ ] DOCX demo fixture review set 구성
  - Fixture set:
    - plain text
    - styles
    - lists
    - tables
    - images
    - multi-section
    - encrypted
- [ ] DOCX MVP acceptance pass
  - Acceptance checks:
    - document opens
    - page rendering works
    - search works
    - text selection works
    - encrypted document asks for password

## Phase 2: PPTX MVP
### PPTX parse layer
- [ ] PPTX slide tree parser 구현
- [ ] slide master/layout/theme link parser 구현
- [ ] text box parser 구현
- [ ] basic shape parser 구현
- [ ] image parser 구현
- [ ] notes and animation exclusion handling 구현

### PPTX layout and interaction
- [ ] slide render model 구현
- [ ] text layout in slide coordinates 구현
- [ ] slide search index 구현
- [ ] slide text selection metadata 구현

### PPTX tests and acceptance
- [ ] Rust fixture tests for slides, themes, shapes, images
- [ ] Flutter slide rendering widget tests
- [ ] PPTX MVP acceptance pass
  - Acceptance checks:
    - slide render
    - text search
    - text selection

## Phase 3: XLSX MVP
### XLSX parse layer
- [ ] workbook and worksheet parser 구현
- [ ] shared strings parser 구현
- [ ] row/column metrics parser 구현
- [ ] cell style subset parser 구현
- [ ] merged cells parser 구현
- [ ] frozen panes parser 구현
- [ ] formula cell cached value parser 구현
  - Note: no formula engine in MVP

### XLSX layout and interaction
- [ ] sheet grid render model 구현
- [ ] visible window cell virtualization 초안 구현
- [ ] cell text search index 구현
- [ ] text-only selection metadata 구현

### XLSX tests and acceptance
- [ ] Rust fixture tests for sheets, merges, frozen panes, cached formulas
- [ ] Flutter grid rendering widget tests
- [ ] XLSX MVP acceptance pass
  - Acceptance checks:
    - sheet render
    - search
    - text-only selection
    - cached formula display

## Phase 4: Hardening
- [ ] password flow end-to-end polish
  - Tests:
    - wrong password retry
    - cancel handling
    - unsupported encryption handling
- [ ] FileHandle input support 구현
  - Tests:
    - handle-based open smoke test
    - large file open smoke test
- [ ] cache tuning and memory profiling
  - Tests:
    - repeated page navigation benchmark
    - large fixture stability run
- [ ] fallback font regression pass
  - Tests:
    - platform fixture review set
- [ ] Linux feasibility review
  - Output: decision note added to `PROJECT_PLAN.md`

## Test procedure by work item
각 체크박스를 끝낼 때 아래 순서를 따른다.

1. Rust unit tests 실행
   - `cargo test --manifest-path rust/Cargo.toml`
2. 영향 fixture가 있으면 fixture test 실행
   - `scripts/run_fixtures.sh`
3. Flutter package tests 실행
   - `cd packages/ms_viewer && fvm flutter test`
   - `cd packages/ms_viewer_platform_interface && fvm flutter test`
4. Demo smoke 확인
   - `cd examples/flutter_demo && fvm flutter test`
5. 필요한 경우 수동 확인 결과를 체크박스 아래에 메모

## Progress log rule
체크박스를 완료 처리할 때는 아래 형식으로 커밋 또는 작업 로그에 남긴다.
- what changed
- which tests passed
- which fixtures were used
- remaining known gaps
