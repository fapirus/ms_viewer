# Development Checklist

이 문서는 실제 개발 진행표다.
각 항목은 구현 완료와 테스트 검증이 끝났을 때만 체크한다.

## Working rules
- 큰 기능은 `Rust 엔진 구현 -> 테스트 -> Flutter 연결 -> 테스트` 순서로 진행한다.
- 각 포맷은 `MVP 구현 -> demo 실연동 -> acceptance` 순서로 닫는다.
- 각 단계는 fixture 또는 자동 테스트가 있어야 완료로 본다.
- 체크는 코드와 테스트가 모두 들어간 뒤에만 한다.
- 새 범위가 생기면 이 문서에 먼저 체크박스로 추가한 뒤 작업한다.
- 로컬 `issue/` 폴더는 실제 문서와 시각 비교 스크린샷 분석용으로 사용하고, 원인 고정 후에는 최소 재현 fixture를 `fixtures/regression/`에 추가한다.
- 각 포맷에서 발견한 후속 품질/성능/호환성 debt는 즉시 `Phase 4: Hardening`에 체크박스로 추가하고, 해당 포맷 phase에는 이유와 차단 여부만 남긴다.
- `format_*` crate 내부 MVP 작업은 병렬 가능하지만, `viewer_core`, `viewer_ffi`, `packages/ms_viewer*`, `examples/flutter_demo`를 건드리는 작업은 직렬로 처리한다.
- demo app 실연동 phase는 공유 표면이 크므로 서로 병렬 진행하지 않는다.
- 병렬 브랜치에서 공통 계약 변경이 필요해지면 작은 통합 커밋을 `develop`에 먼저 반영한 뒤 각 phase 브랜치가 이를 따라간다.

## Parallel work boundary
### 병렬 가능 구간
- `rust/crates/format_pptx/`와 그 하위 테스트
- `rust/crates/format_xlsx/`와 그 하위 테스트
- 포맷별 fixture 추가: `fixtures/pptx/`, `fixtures/xlsx/`, `fixtures/regression/`의 포맷 전용 케이스

### 직렬 처리 구간
- `rust/crates/viewer_core/`
- `rust/crates/viewer_ffi/`
- `packages/ms_viewer_platform_interface/`
- `packages/ms_viewer/`
- `examples/flutter_demo/`
- 공통 문서와 체크리스트: `DEVELOPMENT_CHECKLIST.md`, `PROJECT_PLAN.md`, `docs/architecture/`

### 병렬 작업 규칙
- `PPTX`, `XLSX`는 포맷 전용 parser/layout/search 모델 단계까지는 병렬 진행 가능하다.
- render model, FFI contract, Flutter controller/widget, demo app을 건드리는 시점부터는 직렬 구간으로 전환한다.
- 공통 파일 수정이 필요하면 해당 작업은 별도 통합 커밋으로 잘게 나눠 먼저 병합한다.

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
- [x] DOCX demo fixture review set 구성
  - Done: review fixture corpus and encrypted stub added under fixtures/docx and fixtures/encrypted
  - Fixture set:
    - plain text
    - styles
    - lists
    - tables
    - images
    - multi-section
    - encrypted
- [x] DOCX MVP acceptance pass
  - Done: fixture-backed acceptance tests and fixture runner coverage added
  - Acceptance checks:
    - document opens
    - page rendering works
    - search works
    - text selection works
    - encrypted document asks for password

## Phase 1.5: DOCX demo real integration
### Rust and FFI
- [x] DOCX page render model FFI endpoint 구현
  - Done: viewer_ffi crate split, real docx page count, getPageRenderModel FFI request/response, first page fetch path
  - Rust scope:
    - document session open result에 실제 page count 반영
    - `getPageRenderModel(documentId, pageIndex)` 또는 동등한 FFI 추가
    - page render model 직렬화
  - Tests:
    - first page fetch fixture test
    - invalid page index error mapping test
- [x] DOCX search and selection FFI endpoint 구현
  - Done: searchDocument/getSelectionPage FFI request-response, JSON entrypoint, Dart contract decode
  - Rust scope:
    - search result fetch
    - selection metadata page fetch
  - Tests:
    - search query round-trip test
    - selection metadata fetch smoke test

### Flutter bridge
- [x] Flutter platform bridge에서 DOCX page fetch 연결
  - Done: controller first-page auto fetch, page loading/error/render state split, widget error rendering
  - Flutter scope:
    - document open 후 page fetch
    - loading/error/page state 분리
  - Tests:
    - page fetch controller test
    - page fetch error state widget test
- [x] Flutter search/selection bridge 연결
  - Done: search request/consume, selection page fetch on result jump, drag selection highlight overlay
  - Flutter scope:
    - search result request/consume
    - selection overlay consume
  - Tests:
    - search action integration widget test
    - selection overlay integration widget test

### Demo app
- [x] demo fixture 목록에서 실제 DOCX 열기 연결
  - Done: fixture asset bytes -> viewer_cli -> viewer_ffi open/page fetch path, demo smoke now verifies actual open call
  - Demo scope:
    - bundled fixture를 실제 engine open path로 연결
    - first page render 확인
  - Tests:
    - fixture open smoke test
- [x] demo file picker DOCX 실연동
  - Done: picked docx -> OpenDocumentSource.path real open path, widget-injected picker smoke covers engine path open
  - Demo scope:
    - picked `.docx`를 실제 engine으로 open
    - password-required 오류 표시
  - Tests:
    - picked docx open smoke test
- [x] demo desktop drop DOCX 실연동
  - Done: dropped docx now uses same real path open flow as picked docx, smoke covers dropped path open
  - Demo scope:
    - dropped `.docx`를 실제 engine으로 open
  - Tests:
    - drop docx open smoke test
- [x] DOCX demo real integration acceptance pass
  - Acceptance checks:
    - fixture docx opens through real engine path
    - picked docx opens through real engine path
    - dropped docx opens through real engine path
    - first page render matches real model
    - encrypted docx asks for password in demo flow

## Phase 1.6: DOCX visual parity pass
### Visual regression triage
- [x] issue 기반 DOCX 시각 회귀 분류 규칙 정리
  - Scope:
    - `issue/word/*` 기준으로 페이지 분할, 표, 이미지, 간격, 폰트 차이를 분류
    - 각 이슈는 원인 가설과 재현 조건을 남기고 최소 재현 fixture 후보를 뽑는다
  - Done:
    - 우선순위 테이블 작성
    - 회귀 방지용 최소 fixture 후보 확정
    - `docs/qa/DOCX_VISUAL_TRIAGE.md`에 현재 실문서 분류 결과 반영

### Pagination correctness
- [x] DOCX 페이지 단위 계산 보정
  - Rust scope:
    - paragraph spacing, explicit break, section transition, carry-over height 계산 보정
    - 페이지 끝 줄/블록 누락 방지
  - Tests:
    - multi-page real-world regression fixture
    - page boundary carry-over regression test
  - Done:
    - `w:lastRenderedPageBreak`를 hard page boundary로 해석
    - `fixtures/regression/docx_rendered_page_break.docx` 추가
    - path-based regression test와 synthetic page-break regression test 추가

### Table layout and media
- [x] DOCX 표 크기와 셀 내부 줄바꿈 보정
  - Rust scope:
    - tblGrid, preferred width, cell padding, row height, nested paragraph spacing 반영
    - 셀 내부 이미지/텍스트의 폭 기준 줄바꿈과 높이 계산 보정
  - Tests:
    - real-world table regression fixture
    - table cell wrap regression test
  - Done:
    - 셀 내부 문단을 line 단위로 배치하도록 테이블 레이아웃 경로 정리
    - 셀 텍스트에 실제 run style/font size를 반영
    - `fixtures/regression/docx_table_cell_layout.docx` 추가
    - synthetic table wrap test와 path-based regression test 추가
- [x] DOCX 표 내부 이미지 및 inline image 렌더 보정
  - Rust scope:
    - drawing extent, anchor/inline 차이, cell clipping, image fit 정책 보정
  - Flutter scope:
    - embedded image decode/render regression 방지
  - Tests:
    - image-in-table regression fixture
    - inline image sizing widget test
  - Done:
    - 표 셀 내부 `w:drawing`을 media-aware path로 파싱
    - table cell image node를 page render model에 포함
    - `fixtures/regression/docx_table_inline_image.docx` 추가
    - synthetic image-in-table test, path-based regression test, Flutter image sizing widget test 추가
- [x] DOCX floating table positioning 보정
  - Rust scope:
    - `w:tblpPr`, `w:tblW`, `w:jc` 기반 float/center/preferred width 반영
    - floating table이 inline flow 전체 폭을 점유하지 않도록 레이아웃 분리
  - Tests:
    - floating table metadata parser test
    - centered floating table layout regression fixture
  - Done:
    - `TableLayout`/`FloatingTablePosition` 모델 추가
    - centered floating table의 실제 bounds를 `page render model`에서 회귀 검증
    - `fixtures/regression/docx_floating_table_intro.docx` 추가

### Typography and spacing
- [x] DOCX 문단 간격과 기본 스타일 메트릭 보정
  - Rust scope:
    - `before/after`, line spacing, default paragraph style, section defaults 반영
  - Tests:
    - paragraph spacing regression fixture
- [x] DOCX 문단 정렬과 화면 재개행 보정
  - Rust scope:
    - `w:jc` 문단 정렬을 style/default/direct formatting 경로에서 해석
    - centered/right aligned paragraph의 실제 line x 좌표 보정
    - table cell 내부 문단 정렬도 동일 규칙 적용
  - Flutter scope:
    - engine이 이미 나눈 `TextNode`를 화면에서 다시 줄바꿈하지 않도록 painter 보정
  - Tests:
    - centered paragraph layout regression test
    - single-line render painter regression test
- [x] DOCX inline image 정렬과 table image 페이지 수용량 보정
  - Rust scope:
    - image-only paragraph가 paragraph alignment를 유지하도록 image block에 정렬 정보 반영
    - table cell vertical padding을 보정해서 near-boundary image row가 불필요하게 다음 페이지로 밀리지 않도록 조정
  - Tests:
    - centered inline image layout regression test
    - 3x3 image table single-page regression test
- [x] DOCX 폰트 메트릭과 fallback 정밀도 보정
  - Rust scope:
    - 문자폭 추정 개선 또는 실제 폰트 메트릭 연동 검토
    - CJK/Latin 혼합 문단 폭 계산 보정
  - Tests:
    - mixed script width regression fixture
    - CJK line break regression fixture

### Acceptance
- [x] DOCX visual parity acceptance pass
  - Acceptance checks:
    - 주요 issue 문서가 빈 페이지 없이 렌더된다
    - 실제 Word 대비 페이지 분할이 허용 범위 내에 있다
    - 표 크기와 셀 내부 줄바꿈이 허용 범위 내에 있다
    - 표 내부 이미지와 inline 이미지가 placeholder 없이 렌더된다
    - 최소 재현 fixture 회귀 테스트가 추가되었다

## Phase 2: PPTX MVP
### MVP scope note
- 이 phase는 가능한 한 `format_pptx`와 포맷 전용 fixture 내부에서 닫는다.
- shared render model 또는 공통 text metrics 변경이 필요해지면 즉시 작업을 멈추고 `develop` 기준 공통 통합 작업으로 전환한다.

### PPTX parse layer
- [x] PPTX slide tree parser 구현
  - Done:
    - package root에서 `presentation.xml` 진입점 탐색
    - `sldIdLst` 기반 slide 순서와 slide id 파싱
    - `presentation.xml.rels`에서 slide part target 해석
    - presentation size와 external relationship 예외 처리
  - Tests:
    - minimal slide tree fixture
    - missing slide relationship fixture
    - external relationship handling fixture
- [x] slide master/layout/theme link parser 구현
  - Done:
    - `presentation.xml`의 `sldMasterIdLst` 파싱
    - `presentation -> slideMaster`, `slideMaster -> slideLayout/theme` 관계 해석
    - `slide -> slideLayout` 링크 해석
    - missing master relationship validation 추가
  - Tests:
    - slide to layout relationship fixture
    - master to layout/theme relationship fixture
    - missing master relationship fixture
- [x] text box parser 구현
  - Done:
    - `p:sp -> p:txBody -> a:p/a:r/a:t` 경로 파싱
    - text box bounds와 placeholder type 파싱
    - paragraph alignment/level과 run style subset 파싱
    - line break와 field text 파싱
  - Tests:
    - text box placeholder and bounds fixture
    - run style parsing fixture
    - invalid transform geometry fixture
- [x] basic shape parser 구현
  - Done:
    - `p:sp/p:spPr` 기반 preset geometry 파싱
    - shape transform의 bounds/rotation/flip 파싱
    - solid fill, noFill, line stroke subset 파싱
    - text-only placeholder shape는 visual shape 목록에서 제외
  - Tests:
    - preset geometry with fill/stroke fixture
    - rotation/flip transform fixture
    - invalid stroke color fixture
- [x] image parser 구현
  - Done:
    - `p:pic` 기반 embedded/external image relationship 파싱
    - image transform과 display size 파싱
    - `srcRect` crop subset 파싱
    - description/name fallback과 content type 추론 추가
  - Tests:
    - embedded image relationship fixture
    - external linked image fixture
    - missing image relationship fixture
- [x] notes and animation exclusion handling 구현
  - Done:
    - `notesSlide` relationship 파싱과 slide metadata 연결
    - slide `transition` 존재 여부 추출
    - `timing` subtree element count를 ignored animation metadata로 기록
    - text box parser에서 `timing` subtree를 렌더 파싱 대상에서 제외
  - Tests:
    - notes relationship fixture
    - transition and animation metadata fixture
    - timing subtree exclusion fixture

### PPTX layout and interaction
- [x] slide render model 구현
  - Done:
    - presentation size를 slide page size로 변환
    - shape를 `BoxNode`로 변환
    - embedded image를 `ImageNode`와 base64 payload로 변환
    - text box paragraph를 line node로 배치하는 초기 render model 추가
  - Tests:
    - render model contains text, image, box nodes fixture
    - non-hex scheme color normalization fixture
- [x] text layout in slide coordinates 구현
  - Done:
    - `bodyPr` inset 파싱과 text box content frame 반영
    - paragraph level 기반 indent 반영
    - token/character 단위 wrap 추가
    - box 높이 경계 안에서 line 배치
    - centered/right paragraph의 line x 계산 보강
  - Tests:
    - centered title line positioning fixture
    - narrow body text multi-line wrap fixture
- [x] slide search index 구현
  - Done:
    - slide 단위 `SearchPage` 생성
    - text box paragraph/run 텍스트를 검색용 문자열로 평탄화
    - notes/animation subtree를 제외한 본문 슬라이드 텍스트만 인덱싱
    - 대소문자 무시 검색 매치 생성
  - Tests:
    - multi-slide search page fixture
    - case-insensitive slide search fixture
- [x] slide text selection metadata 구현
  - Done:
    - `PageRenderModel.selectionAnchors` 생성
    - slide text span별 `TextRange` 연속 offset 보장
    - 줄바꿈된 multi-line text node에 대해 char-level anchor 생성
  - Tests:
    - single-line title anchor fixture
    - wrapped body text range continuity fixture

### PPTX tests and acceptance
- [x] Rust fixture tests for slides, themes, shapes, images
  - Done:
    - `fixtures/pptx/` review fixture set 추가
    - 실제 `.pptx` 파일을 여는 acceptance test 추가
    - slides/shapes/search-selection fixture와 theme/layout/image fixture를 분리
  - Tests:
    - fixture presence smoke test
    - review fixture open/render/search/selection test
    - review fixture theme/layout/image link test
- [x] Flutter slide rendering widget tests
  - Done:
    - `DocumentPageView`에서 PPTX slide의 shape/text/image layer 조합 위젯 테스트 추가
    - `MsDocumentView`에서 PPTX 첫 페이지 로드와 navigation shell 위젯 테스트 추가
  - Tests:
    - embedded image layer widget test
    - PPTX page preview and next-page navigation widget test
- [x] PPTX MVP acceptance pass
  - Acceptance checks:
    - slide render
    - text search
    - text selection
  - Verified by:
    - `acceptance_pptx_review_set` review fixture render/search/selection 통과
    - Flutter slide preview/navigation widget tests 통과

## Phase 2.5: PPTX demo real integration
### Serialized integration gate
- 이 phase는 `viewer_ffi`, `packages/ms_viewer*`, `examples/flutter_demo`를 건드린다.
- 다른 포맷 demo phase와 병렬 진행하지 않는다.
- 권장 브랜치 전략: `PPTX MVP` 브랜치와 별도 `PPTX demo integration` 브랜치로 분리한다.

### Rust and FFI
- [x] PPTX slide render model FFI endpoint 연결
  - Done:
    - `viewer_ffi`에서 문서 kind를 감지해 PPTX는 `build_slide_render_model(...)`로 slide fetch 분기
    - DOCX/XLSX 경로와 충돌하지 않도록 source -> archive helper 분리
  - Tests:
    - first slide fetch fixture test
    - invalid slide index error mapping test
- [x] PPTX search and selection FFI endpoint 연결
  - Done:
    - `viewer_ffi`에서 PPTX search를 `search_slides(...)`로 연결
    - `get_selection_page`가 PPTX slide selection metadata를 반환하도록 page fetch 경로 재사용
    - `OpenDocumentSuccess.capabilities`에 PPTX search/selection 지원 반영
  - Tests:
    - slide search round-trip test
    - slide selection metadata fetch smoke test

### Flutter bridge
- [x] Flutter platform bridge에서 PPTX slide fetch 연결
  - Done:
    - `MsViewerController.openDocument(...)`가 PPTX open 성공 시 첫 slide를 자동 fetch하도록 확장
    - PPTX preview/navigation widget test를 자동 first-slide fetch 흐름에 맞춰 정리
    - PPTX slide fetch error state widget test 추가
  - Tests:
    - slide fetch controller test
    - slide fetch error state widget test
- [x] Flutter PPTX search/selection bridge 연결
  - Done:
    - 기존 포맷 중립 search/selection 흐름이 PPTX에서도 동작함을 widget test로 고정
    - PPTX slide search result tap -> selection page fetch -> highlight 적용 흐름 검증
    - PPTX slide drag selection highlight 흐름 검증
  - Tests:
    - slide search integration widget test
    - slide selection integration widget test

### Demo app
- [x] demo fixture 목록에서 실제 PPTX 열기 연결
  - Done:
    - demo fixture catalog에 `pptx_text_shapes.pptx`, `pptx_theme_layout_images.pptx` 추가
    - fixture PPTX를 asset으로 등록하고 선택 시 `bytesBase64` source로 실제 engine open 연결
    - demo smoke test에 fixture PPTX open 경로 추가
- [x] demo file picker PPTX 실연동
  - Done:
    - picked `.pptx` import가 실제 engine path open을 타도록 분기 확장
    - picked `.pptx` smoke test 추가
- [x] demo desktop drop PPTX 실연동
  - Done:
    - dropped `.pptx` import가 실제 engine path open을 타도록 분기 확장
    - dropped `.pptx` smoke test 추가
- [x] PPTX demo real integration acceptance pass
  - Acceptance checks:
    - fixture pptx opens through real engine path
    - picked pptx opens through real engine path
    - dropped pptx opens through real engine path
    - first slide render matches real model
  - Done:
    - `fixday` 및 review fixture 기준으로 desktop demo의 fixture/picked/dropped 경로를 수동 확인
    - first slide render가 실제 engine render model과 일치하는 수준으로 확인됨

## Phase 2.6: PPTX visual parity pass
### Visual regression triage
- [x] issue 기반 PPTX 시각 회귀 분류 규칙 정리
  - Scope:
    - slide 배치, 텍스트 박스 정렬, shape/image fit, theme font 차이를 `issue/` 기준으로 분류
    - 최소 재현 fixture 후보를 `fixtures/regression/`에 승격
  - Notes:
    - `fixday` 케이스는 [docs/qa/PPTX_VISUAL_TRIAGE.md](/Users/ultramarine/Documents/Workspace/Fapirus/ms_viewer/docs/qa/PPTX_VISUAL_TRIAGE.md) 기준으로 관리한다

### Rendering completeness
- [x] PPTX placeholder layout inheritance 보정
  - Tests:
    - placeholder without local `spPr` inherits layout bounds
- [x] PPTX group shape, table, background rendering 보정
  - Tests:
    - `grpSp` recursive image render regression
    - `graphicFrame/a:tbl` render regression
    - master background regression
- [x] PPTX 표 셀 배경색과 표 내부 텍스트 스타일 보정
  - Notes:
    - `fixday` slide 1 우하단 표에서 cell fill, border, text run style 일부가 누락된다
    - 표가 읽히는 수준은 넘었지만, 현재는 acceptance를 닫기 어려운 오차다
    - 기본 table style에 font size가 없을 때 18pt fallback으로 셀 텍스트가 잘리는 문제가 있어, cell height 기반 기본 font size와 vertical centering을 반영했다
  - Tests:
    - table cell fill regression fixture
    - table rich-text run style regression fixture
    - default table style fallback regression fixture
- [x] PPTX full-slide background panel 곡률 보정
  - Notes:
    - `fixday` slide 1의 레이아웃 배경처럼 slide 전체를 덮는 `roundRect`는 그대로 곡률을 주면 PDF보다 과장된 흰 패널이 된다
    - full-slide background container는 곡률을 제거하고, 실제 콘텐츠용 `roundRect`만 곡률을 유지한다
  - Tests:
    - full-slide background round-rect regression fixture
- [x] PPTX repaint flicker triage 및 hardening 이관
  - Notes:
    - `gaplessPlayback` 1차 적용
    - 수동 점검에서 잔존 가능성이 있으나 문서 열람을 막는 수준은 아니므로 `Phase 4: Hardening`으로 이관

### Geometry and typography
- [x] PPTX 텍스트 박스 정렬과 줄바꿈 보정
  - Tests:
    - centered title regression fixture
    - mixed font line break regression fixture
    - bullet/default paragraph indent regression fixture
- [x] PPTX 밑줄, 텍스트 clipping, 제목/본문 개행 보정
  - Notes:
    - `fixday` slide 2~5에서 underline이 빠지고 `Client -> Clien` clipping이 발생한다
    - 제목 영역의 띄어쓰기/개행도 여전히 PDF와 차이가 있어 acceptance 전 보정이 필요하다
  - Tests:
    - underline text render regression fixture
    - last-glyph clipping regression fixture
    - mixed Korean/Latin title wrapping regression fixture
    - `wrap=\"none\"` text box regression fixture
- [x] PPTX text box vertical anchor 보정
  - Notes:
    - `fixday` slide 5~6은 `bodyPr anchor=\"ctr\"`를 많이 사용하지만, 엔진은 상단 기준으로만 텍스트를 배치하고 있었다
    - 카드/오버레이 내부 텍스트 배치가 어긋나는 핵심 원인이므로 visual parity 전에 반영한다
  - Tests:
    - `anchor=\"ctr\"` parser regression fixture
    - centered card vertical placement regression fixture
- [x] PPTX shape/image transform 및 crop 보정
  - Tests:
    - image crop regression fixture
    - rotated shape bounds regression fixture
- [x] PPTX shape z-order, opacity, rounded corner, overlay composition 보정
  - Notes:
    - `fixday` slide 2, 5, 6에서 회색 오버레이, 반투명 도형, 흰색 마스크, 곡률, 그림자, 겹침 순서가 PDF와 다르다
    - 원본 노드 순서 보존, alpha, roundRect 곡률, shape style fallback은 이번 라운드에서 반영
    - exact shadow/effect fidelity는 `Phase 4: Hardening`으로 이관
  - Tests:
    - overlay z-order regression fixture
    - translucent shape opacity regression fixture
    - rounded rectangle corner radius regression fixture
- [x] PPTX theme font와 기본 스타일 메트릭 보정
  - Tests:
    - theme font regression fixture
    - line spacing regression fixture
    - gradient title/default fill regression fixture

### Acceptance
- [x] PPTX visual parity acceptance pass
  - Acceptance checks:
    - 주요 issue slide가 빈 화면 없이 렌더된다
    - 텍스트 박스 정렬과 줄바꿈이 허용 범위 내에 있다
    - image/shape 배치가 허용 범위 내에 있다
    - table fill/text, underline, overlay composition이 주요 issue slide에서 재현된다
    - 최소 재현 fixture 회귀 테스트가 추가되었다
  - Done:
    - `issue/powerpoint/fixday` 1~6 페이지 기준으로 구조 이슈가 해소됨
    - 남은 미세 오차는 font metrics, effect, repaint flicker 성격으로 `Phase 4: Hardening`에 이관

## Phase 3: XLSX MVP
### MVP scope note
- 이 phase는 가능한 한 `format_xlsx`와 포맷 전용 fixture 내부에서 닫는다.
- 공통 grid/render/search 계약 변경이 필요해지면 즉시 작업을 멈추고 `develop` 기준 공통 통합 작업으로 전환한다.

### XLSX parse layer
- [x] workbook and worksheet parser 구현
  - Done:
    - package root에서 `xl/workbook.xml` 진입점 탐색
    - workbook sheet 순서, `activeTab`, `date1904`, sheet visibility 파싱
    - `workbook.xml.rels`를 통해 worksheet part target 해석
    - 각 worksheet root와 `dimension` 존재 여부 검증
  - Tests:
    - workbook order and visibility fixture
    - missing worksheet relationship fixture
    - non-worksheet relationship fixture
- [x] shared strings parser 구현
  - Done:
    - `workbook.xml.rels`에서 `sharedStrings.xml` target 해석
    - plain string과 rich-text run 조합을 하나의 shared string으로 평탄화
    - shared strings part가 없는 workbook은 빈 테이블로 처리
  - Tests:
    - simple and rich shared strings fixture
    - missing shared strings relationship fixture
    - missing shared strings part fixture
- [x] row/column metrics parser 구현
  - Done:
    - worksheet `sheetFormatPr`의 `defaultRowHeight`, `defaultColWidth` 파싱
    - `cols/col`의 범위별 width, hidden, customWidth 파싱
    - `sheetData/row`의 row index, height, hidden, customHeight 파싱
  - Tests:
    - default and custom row/column metrics fixture
    - malformed decimal metric fixture
- [x] cell style subset parser 구현
  - Done:
    - `styles.xml` target 해석
    - custom number formats, fonts, fills, `cellXfs` subset 파싱
    - horizontal/vertical alignment와 `wrapText` 최소 subset 반영
    - styles part가 없을 때 empty catalog 반환
  - Tests:
    - styles part subset fixture
    - missing styles relationship fixture
    - invalid alignment fixture
- [x] merged cells parser 구현
  - Done:
    - worksheet `mergeCells/mergeCell` range 파싱
    - `A1:C3` 형태 ref를 row/column 좌표로 정규화
    - 잘못된 범위 순서나 malformed ref는 즉시 invalid 처리
  - Tests:
    - merged cell ranges fixture
    - worksheet without mergeCells fixture
    - invalid merged range fixture
- [x] frozen panes parser 구현
  - Done:
    - worksheet `sheetViews/sheetView/pane`에서 frozen/frozenSplit state 파싱
    - `xSplit`, `ySplit`, `topLeftCell`, `activePane` 최소 subset 보존
    - split pane은 viewport 범위 밖으로 보고 무시, malformed state/cell ref는 invalid 처리
  - Tests:
    - frozen rows and columns fixture
    - non-frozen split pane fixture
    - invalid pane state fixture
- [x] formula cell cached value parser 구현
  - Done:
    - worksheet `sheetData/row/c` subset 파싱
    - formula 문자열과 cached value를 분리해서 보존
    - shared string, inline string, boolean, numeric, error 최소 타입 지원
    - formula engine은 여전히 미구현이며 cached value만 사용
  - Tests:
    - formula cells with cached values fixture
    - formula without cached value fixture
    - invalid shared string index fixture

### XLSX layout and interaction
- [x] sheet grid render model 구현
  - Done:
    - worksheet cell subset, metrics, merges, style subset을 공통 `PageRenderModel`로 투영
    - 셀을 `BoxNode + TextNode`로 렌더하고 merged cell span을 단일 box로 처리
    - cached formula value, boolean, error, shared/inline string을 표시값으로 반영
    - text-only selection anchor를 셀 텍스트 기준으로 생성
  - Tests:
    - merged header + cached formula + style render model fixture
- [x] visible window cell virtualization 초안 구현
  - Done:
    - row/column window 기준으로 worksheet grid를 부분 렌더하는 초안 추가
    - visible window 좌표계를 local origin으로 재정렬
    - window 범위를 벗어난 셀은 제외하고 향후 frozen/merged edge case는 보정 phase로 이관
  - Tests:
    - 2x2 visible window slice fixture
- [x] cell text search index 구현
  - Done:
    - worksheet cell 값을 row-major text로 평탄화해서 시트 단위 `SearchPage` 생성
    - cached formula value, boolean, inline/shared string을 검색 인덱스에 포함
    - 공통 case-insensitive search matcher 재사용
  - Tests:
    - sheet search pages fixture
    - case-insensitive workbook search fixture
- [x] text-only selection metadata 구현
  - Done:
    - 공통 `PageRenderModel.selectionAnchors`를 시트 단위 selection helper로 노출
    - 텍스트 노드에 대해서만 char-level anchor 생성
    - 숫자/문자열/cached formula 결과를 모두 text-only selection 대상으로 포함
  - Tests:
    - text-only selection anchor fixture

### XLSX tests and acceptance
- [x] Rust fixture tests for sheets, merges, frozen panes, cached formulas
  - Done:
    - 실제 review fixture 2종을 `fixtures/xlsx/`에 추가
    - workbook/sheet open, merged cells, frozen panes, cached formula, render/search/selection acceptance를 실제 파일 기준으로 검증
  - Tests:
    - fixture presence smoke test
    - acceptance_xlsx_review_set
- [x] Flutter grid rendering widget tests
  - Done:
    - 공통 `DocumentPageView`가 XLSX 스타일의 dense grid page model을 그리는지 검증
    - `MsDocumentView` preview shell이 XLSX preview page를 소비하는지 검증
  - Tests:
    - xlsx grid page widget test
    - xlsx preview shell widget test
- [x] XLSX MVP acceptance pass
  - Acceptance checks:
    - sheet render
    - search
    - text-only selection
    - cached formula display
  - Done:
    - render/search/selection/cached formula가 실제 review fixture와 Flutter widget smoke에서 모두 검증됨

## Phase 3.5: XLSX demo real integration
### Serialized integration gate
- 이 phase는 `viewer_ffi`, `packages/ms_viewer*`, `examples/flutter_demo`를 건드린다.
- 다른 포맷 demo phase와 병렬 진행하지 않는다.
- 권장 브랜치 전략: `XLSX MVP` 브랜치와 별도 `XLSX demo integration` 브랜치로 분리한다.

### Rust and FFI
- [x] XLSX visible sheet window FFI endpoint 연결
  - Tests:
    - first sheet window fetch fixture test
    - invalid sheet index error mapping test
- [x] XLSX search and selection FFI endpoint 연결
  - Tests:
    - sheet search round-trip test
    - sheet selection metadata fetch smoke test

### Flutter bridge
- [x] Flutter platform bridge에서 XLSX sheet window fetch 연결
  - Tests:
    - sheet window fetch controller test
    - sheet fetch error state widget test
- [x] Flutter XLSX search/selection bridge 연결
  - Tests:
    - sheet search integration widget test
    - sheet text selection integration widget test

### Demo app
- [x] demo fixture 목록에서 실제 XLSX 열기 연결
- [x] demo file picker XLSX 실연동
- [x] demo desktop drop XLSX 실연동
- [ ] XLSX demo real integration acceptance pass
  - Acceptance checks:
    - fixture xlsx opens through real engine path
    - picked xlsx opens through real engine path
    - dropped xlsx opens through real engine path
    - visible sheet window render matches real model

## Phase 3.55: Demo viewer shell refactor
### Purpose
- 이 phase는 `examples/flutter_demo`와 `packages/ms_viewer`의 viewer shell을 실제 제품에 가까운 구조로 재편한다.
- `XLSX visual parity acceptance` 전에 수행한다.
- `DOCX/PPTX/XLSX` 수동 검증 기준을 preview shell이 아니라 document viewer shell 기준으로 바꾼다.

### Structure
- [x] 문서 목록 화면과 뷰어 화면 분리
  - Done when:
    - library screen과 viewer screen이 route 단위로 분리된다
    - 문서 선택 시 viewer route로 이동한다
  - Tests:
    - library -> viewer navigation widget test
    - back navigation widget test
- [x] viewer 전용 chrome 구성
  - Scope:
    - title, back, search, open file를 viewer 화면 기준으로 재배치
    - preview shell 성격의 보조 메타 패널 제거
  - Tests:
    - viewer app bar widget test
    - search placement widget test
- [x] DOCX continuous page scroll viewer 적용
  - Scope:
    - engine page model은 유지
    - Flutter UI는 page button paging 대신 vertical scroll stack으로 전환
  - Tests:
    - docx continuous scroll widget test
    - page stack smoke test
- [x] PPTX continuous slide scroll viewer 적용
  - Scope:
    - engine slide index는 유지
    - Flutter UI는 slide button paging 대신 vertical slide list로 전환
  - Tests:
    - pptx continuous scroll widget test
    - slide list smoke test
- [x] XLSX full-screen viewport shell 적용
  - Scope:
    - side rail과 preview layout 없이 sheet viewport를 문서 전용 화면으로 제공
    - row/column header와 2D viewport를 주 surface로 둔다
  - Tests:
    - xlsx full-screen shell widget test
    - viewport sizing smoke test
- [x] demo viewer shell acceptance pass
  - Acceptance checks:
    - demo 첫 화면은 library 역할만 한다
    - 열람 시 별도 viewer screen으로 이동한다
    - `DOCX/PPTX`는 scroll viewing으로 동작한다
    - `XLSX`는 spreadsheet viewer처럼 보인다
    - 이후 issue screenshot은 viewer screen 기준으로 수집한다

## Phase 3.6: XLSX visual parity and large-sheet pass
### Visual regression triage
- [x] XLSX grid-first viewer UX 아키텍처 확정
  - Done when:
    - `docs/architecture/xlsx_viewer_ux.md` 기준으로 UX 방향이 고정된다
    - `XLSX`가 page viewer가 아니라 sheet viewport라는 점이 `PROJECT_PLAN.md`와 동기화된다
  - Output:
    - row/column header, 2D scroll, frozen pane, visible window 정책 확정
- [ ] issue 기반 XLSX 시각 회귀 분류 규칙 정리
  - Scope:
    - column width, row height, merged cell, frozen pane, large-sheet viewport 차이를 `issue/` 기준으로 분류
    - 최소 재현 fixture 후보를 `fixtures/regression/`에 승격

### Layout and viewport fidelity
- [x] Flutter 전용 2D sheet viewport scaffold 구현
  - Scope:
    - 공통 `DocumentPageView` 대신 `XLSX` 전용 viewport shell 도입
    - `TableView` 또는 동등한 2D viewport 기반으로 row/column scrolling 구조 구성
  - Tests:
    - initial viewport widget test
    - 2D scroll smoke test
- [x] XLSX 최소 visible window와 기본 셀 수 확장 보정
  - Scope:
    - 첫 렌더에서 너무 적은 셀만 보이지 않도록 minimum row/column budget 적용
    - viewport 크기에 따라 초기 visible window를 한 번 확장한다
  - Tests:
    - minimum visible window expansion widget test
- [x] pinned row/column headers와 corner cell 구현
  - Tests:
    - pinned header widget test
    - header/body scroll sync test
- [x] XLSX column width/row height/merged cell 배치 보정
  - Tests:
    - merged cell layout regression fixture
    - row height regression fixture
- [x] XLSX frozen pane와 visible window virtualization 보정
  - Tests:
    - frozen pane viewport regression fixture
    - large-sheet scroll stability regression fixture
- [x] XLSX effective bounds와 overscan 정책 보정
  - Scope:
    - `dimension`, actual cells, metrics, merges, frozen panes를 합쳐 effective bounds 계산
    - global max row/column가 아니라 used-range 중심 viewport를 사용
  - Tests:
    - effective bounds regression fixture
    - overscan stability regression fixture
- [x] XLSX number/date format 및 기본 타이포그래피 보정
  - Tests:
    - number/date display regression fixture
    - mixed width text regression fixture
- [x] XLSX sheet tab navigation shell 구현
  - Scope:
    - `next/previous` 대신 workbook sheet name 기반 하단 탭을 사용한다
    - hidden sheet는 탭에 노출하지 않고 active sheet를 우선 연다
  - Tests:
    - open contract sheet tab decode test
    - xlsx tab switch widget test

### Acceptance
- [ ] XLSX visual parity acceptance pass
  - Acceptance checks:
    - 주요 issue sheet가 빈 영역/잘린 영역 없이 렌더된다
    - `XLSX`가 page viewer처럼 보이지 않고 spreadsheet viewport처럼 동작한다
    - row/column headers가 유지된다
    - 하단 시트 탭으로 workbook sheet 전환이 가능하다
    - column width와 row height가 허용 범위 내에 있다
    - frozen pane과 visible window가 안정적으로 동작한다
    - 최소 재현 fixture 회귀 테스트가 추가되었다

## Phase 4: Hardening
- [ ] deferred hardening backlog sweep
  - Scope:
    - `DOCX/PPTX/XLSX` phase에서 미룬 품질/성능/호환성 debt를 전수 검토
    - 중복 항목 정리와 우선순위 재배치
  - Output:
    - `PROJECT_PLAN.md`와 이 문서의 hardening backlog 동기화
- [ ] password flow end-to-end polish
  - Tests:
    - wrong password retry
    - cancel handling
    - unsupported encryption handling
- [ ] DOCX theme font 해석과 exact glyph metrics 보강
  - Notes:
    - Word theme/default font 해석과 실제 glyph metrics 기반 line break는 시각 충실도 후속 과제
    - 현재는 fallback/font-family heuristic 기반으로 렌더링
  - Tests:
    - issue screenshot review set
    - theme font fixture regression
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
- [ ] cross-format visual parity backlog pass
  - Scope:
    - DOCX/PPTX/XLSX에서 남긴 시각 충실도 잔여 항목을 공통 정책과 포맷별 정책으로 다시 나눠 처리
  - Tests:
    - issue screenshot review set
- [ ] PPTX exact glyph metrics, font spacing, line-wrap fidelity 보강
  - Notes:
    - `fixday` slide 1~6 비교 기준으로 남은 띄어쓰기, 글자폭, 줄바꿈 오차는 구조 버그보다 폰트 메트릭 오차 성격이 강하다
    - acceptance를 막는 구조 이슈를 먼저 닫고, 최종 경화 단계에서 exact glyph metrics와 서체 availability를 같이 본다
  - Tests:
    - issue screenshot review set
    - mixed Korean/Latin glyph width regression fixture
    - representative real-world review documents
- [ ] PPTX repaint flicker 최종 완화
  - Notes:
    - `gaplessPlayback` 1차 적용 후에도 데모 앱 상호작용 시 잔존 가능성이 있다
    - visual parity acceptance는 통과했지만, 최종 품질 경화 단계에서 repaint 경로를 다시 점검한다
  - Tests:
    - desktop demo interaction smoke
    - repeated page navigation smoke
- [ ] PPTX shadow/effect fidelity 보강
  - Notes:
    - slide 5~6의 outer shadow, effectRef 기반 표현은 핵심 배치보다 후순위로 미뤘다
    - 현재는 z-order, opacity, 곡률, crop/flip까지 맞춘 상태이며 effect 픽셀 정밀도는 hardening에서 보정
  - Tests:
    - issue screenshot review set
    - shadow/effect smoke regression fixture
- [ ] coverage tooling and threshold 정리
  - Scope:
    - Rust coverage 도구 도입
    - Flutter package coverage threshold 기준 고정
  - Output:
    - coverage 실행 명령과 기준 문서화
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
5. demo 실연동 phase라면 실제 fixture open 경로 수동 확인
6. 필요한 경우 수동 확인 결과를 체크박스 아래에 메모

## Progress log rule
체크박스를 완료 처리할 때는 아래 형식으로 커밋 또는 작업 로그에 남긴다.
- what changed
- which tests passed
- which fixtures were used
- remaining known gaps
