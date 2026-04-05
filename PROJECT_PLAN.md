# MS Viewer Project Plan

## Project goal
Flutter에서 사용할 수 있는 Microsoft Office 문서 뷰어를 만든다.
핵심 엔진은 Rust로 구현하고, Flutter와는 FFI로 연결한다.
최종적으로 Flutter 앱에서 `docx`, `pptx`, `xlsx` 문서를 안정적으로 렌더링하고 탐색할 수 있어야 한다.

## Product direction
- 1차 목표는 `editing`이 아니라 `viewing`이다.
- 뷰어 품질을 먼저 확보하고, 편집 기능은 범위에서 제외한다.
- 개발 순서는 `docx -> pptx -> xlsx`다.
- 각 포맷은 `MVP 구현 -> demo 실연동 -> 다음 포맷` 순서로 진행한다.
- 정확도와 안정성을 성능보다 조금 더 우선한다.
- 각 포맷에서 발견한 후속 품질/성능/호환성 문제는 즉시 hardening backlog로 승격한다.

## Parallel development policy
병렬 작업은 가능하지만, 범위를 잘못 잡으면 공유 레이어 충돌이 커진다.

### Parallel-safe scope
- `rust/crates/format_docx/`, `rust/crates/format_pptx/`, `rust/crates/format_xlsx/`의 포맷 전용 parser/layout/search 구현
- 포맷 전용 fixture와 regression fixture 추가
- 포맷 전용 테스트 코드

### Serialized scope
- `rust/crates/viewer_core/`
- `rust/crates/viewer_ffi/`
- `packages/ms_viewer_platform_interface/`
- `packages/ms_viewer/`
- `examples/flutter_demo/`
- 공통 계획/아키텍처 문서

### Working rule
- `PPTX`와 `XLSX`의 엔진 MVP는 병렬 진행 가능하다.
- shared render model, FFI contract, Flutter bridge, demo app은 직렬 단계로 본다.
- demo 실연동 phase는 포맷별로 순차 진행한다.
- 공통 계약 변경이 필요하면 작은 통합 커밋을 먼저 `develop`에 반영한 뒤 각 phase 브랜치가 이를 따라간다.
- 권장 브랜치 전략은 `format engine branch -> demo integration branch -> visual parity branch` 순서로 끊는 것이다.

## Target platforms
### Phase 1 targets
- iOS
- Android
- macOS
- Windows

### Secondary target
- Linux: 1차 범위에서는 제외하고, 아키텍처를 재사용할 수 있으면 후속 지원

### Excluded
- Web: FFI 중심 구조와 맞지 않으므로 1차 범위에서 제외

## Core features
- 문서 열람
- 검색
- 텍스트 선택
- 비밀번호 보호 문서 열람
- `FileHandle` 기반 입력 지원

## Non-goals
- 문서 편집
- Office 원본과 100% 동일한 편집 호환성
- 초기 단계에서의 모든 애니메이션, 매크로, VBA, 고급 수식 엔진 완전 지원
- Web 지원
- Linux 1차 출시 포함 보장

## Important correction
"Rust 엔진이 Flutter로 문서를 그린다"는 표현은 구현 관점에서 조금 더 구체화할 필요가 있다.
검색과 텍스트 선택이 필요하므로, Rust는 단순 비트맵만 주는 방식보다 아래 중 하나를 반환해야 한다.

- 레이아웃 트리
- 디스플레이 리스트
- 텍스트 런과 좌표 정보가 포함된 렌더 모델

즉, Flutter는 최종 페인팅과 상호작용을 맡고, Rust는 파싱, 레이아웃, 검색 인덱싱, 선택 가능한 텍스트 메타데이터를 제공하는 구조가 더 적절하다.

## Fixed product decisions
- `docx`는 페이지 단위 뷰를 우선한다.
- 검색은 일반적인 웹 검색 수준의 UX를 목표로 한다.
- 텍스트 선택은 순수 텍스트만 지원한다.
- 표 셀 단위 선택은 지원하지 않는다.
- 암호화 문서는 비밀번호 입력 UI까지 제공한다.
- 폰트 정책은 `document font first + curated fallback map`으로 간다.
- `xlsx`는 `cached value only` 정책으로 간다.
- 대용량 문서는 `archive-backed lazy loading`으로 대응한다.
- 테스트는 fixture 중심 전략으로 간다.

## DOCX page model
### Selected
- 페이지 단위 뷰

### Why
- Word 문서 사용자는 페이지 기준 인지에 익숙하다.
- 인쇄형 문서, 헤더/푸터, 섹션, 페이지 나눔, 여백 표현에 유리하다.
- 검색 결과 위치와 선택 범위를 페이지 좌표로 다루기 쉽다.

### Difficulty tradeoff
- 연속 스크롤 뷰보다 난이도는 높다.
- 이유는 레이아웃 시점에 페이지 분할, 페이지 높이, 줄바꿈, 고아/과부 줄, 헤더/푸터 영향을 같이 계산해야 하기 때문이다.
- 하지만 Office 스타일 문서에서는 페이지 모델이 최종 사용자 기대와 더 잘 맞는다.

### Final decision
- `docx`는 처음부터 페이지 모델로 간다.

## Search behavior
웹 검색 수준은 아래처럼 정의한다.
- 기본값은 대소문자 무시
- 일반 문자열 검색 우선
- 정규식 미지원
- 전체 문서 결과 목록 제공
- 현재 매치 하이라이트
- 다음/이전 결과 이동

## Text selection scope
### Selected
- 순수 텍스트 선택만 지원

### Implication
- 문단, 표 안 텍스트, 텍스트 박스 안 텍스트는 선택 가능
- 표 셀 자체 선택, 도형 선택, 이미지 선택은 제외

## Font fallback policy
이 부분은 단순히 "기본 폰트 하나"만 쓰면 자연스러운 결과가 잘 나오지 않는다.
보통 문서 에디터나 뷰어는 아래 방식으로 동작한다.

### Selected approach
- 문서 지정 폰트를 우선 시도
- 플랫폼별 curated fallback map 사용
- 문자 스크립트별 fallback 체인 적용

### Typical editor strategy
1. 문서에 지정된 폰트를 우선 시도
2. OS에 해당 폰트가 있으면 사용
3. 없으면 비슷한 계열의 fallback 폰트로 대체
4. 문자 범위별로 fallback 체인을 적용
   - Latin
   - CJK
   - emoji
   - symbol

### Initial policy
- Windows
  - `Calibri -> Arial`
  - `Cambria -> Times New Roman`
  - `Malgun Gothic -> Malgun Gothic`
- macOS
  - `Calibri -> Helvetica` 또는 `Arial`
  - `Cambria -> Times`
  - `Malgun Gothic -> Apple SD Gothic Neo`
- Android
  - `Calibri -> Roboto`
  - `Cambria -> Noto Serif`
  - `Malgun Gothic -> Noto Sans CJK`
- iOS
  - `Calibri -> San Francisco` 계열 또는 `Arial`
  - `Cambria -> Times New Roman` 계열 serif
  - `Malgun Gothic -> Apple SD Gothic Neo`

### Note
- 정확한 fallback map은 별도 정책 문서로 관리하는 것이 좋다.
- 초기에는 대표 폰트군 기준으로 시작하고, fixture 기반 회귀를 통해 보강한다.

## XLSX formula policy
질문한 `cached value only`는 이 뜻이다.

### What is cached value
Excel 파일에는 셀 수식이 들어갈 수 있다.
예:
- 셀 내용: `=A1+B1`
- 마지막으로 Excel이 계산해 저장한 값: `42`

### Selected approach
- 기본 렌더링은 cached value 우선
- 수식을 직접 다시 계산하지 않음
- 캐시가 없는 수식 셀은 계산 미지원 상태로 표시
- 수식 엔진은 후속 과제로 분리

### Why
- 보기 전용 뷰어의 1차 목표와 잘 맞는다.
- 수식 엔진은 함수, 범위 참조, locale, 날짜 처리까지 포함되므로 별도 프로젝트에 가깝다.

## Encrypted documents
### Selected
- 비밀번호 입력 UI 제공

### UX policy
- Flutter에서 비밀번호 입력 다이얼로그 제공
- 비밀번호는 메모리에만 유지하고 영구 저장하지 않음
- 잘못된 비밀번호는 재입력 가능
- 사용자가 취소하면 명시적 취소 상태 반환
- 지원 불가 암호화 방식은 구분된 오류로 반환

### Architecture implication
- Flutter는 비밀번호 입력 UI 담당
- Rust 엔진은 암호화 감지, 복호화 시도, 실패 원인 반환 담당
- 잘못된 비밀번호, 지원 불가 암호화 방식, 손상 파일을 구분해서 에러 모델을 설계해야 한다.

## Large file and memory strategy
사용자 의도는 "가능하면 모두 메모리에 올리지 말고, 큰 파일도 안정적으로 열고 싶다"로 해석된다.

### Important constraint
OOXML은 ZIP 기반이라 완전한 스트리밍이 쉽지 않다.
- 중앙 디렉터리 읽기 필요
- 여러 XML 파트를 랜덤 접근해야 함
- 이미지/미디어도 별도 파트로 접근해야 함

즉, `100MB+` 문서를 다루려면 "완전 스트리밍"보다 "필요 파트만 지연 로딩"이 현실적이다.

### Selected approach
- ZIP 엔트리 인덱스만 먼저 읽기
- 필요한 파트만 지연 로딩
- XML도 가능한 범위에서 지연 파싱
- 페이지/슬라이드/시트 단위 캐시 사용

### Cache policy recommendation applied
- 현재 보고 있는 페이지/슬라이드/시트를 최우선 유지
- 인접 1~2개 단위 선로딩 허용
- 화면에서 멀어진 render cache는 해제 가능
- 원본 archive 인덱스와 최소 메타데이터는 유지
- 캐시 eviction은 LRU 기반 단순 정책으로 시작

### Why
- 대형 파일 대응과 구현 복잡도의 균형이 가장 좋다.
- 안정성을 우선할 때도 메모리 폭증을 막기 좋다.

## Performance posture
정량 SLA는 아직 두지 않지만 방향은 아래처럼 잡는다.
- 첫 화면은 가능한 빨리
- 전체 문서는 필요 시점에만 파싱
- 성능보다 안정성을 우선
- 큰 파일에서도 크래시 없이 동작하는 것을 우선 목표로 둔다.

## Hardening posture
`docx`, `pptx`, `xlsx`를 한 바퀴 모두 돌고 나면 hardening phase에서 아래를 묶어 처리한다.

- `xlsx` drawing/image, chart placeholder, zoom 같은 남은 sheet-viewer 기능
- 공통 text metrics / fallback / theme font / glyph fidelity 보강
- 암호화/비밀번호 UX polish와 `FileHandle` 입력 support
- 대용량 파일 cache tuning / memory profiling / Linux feasibility review
- cross-format visual parity backlog와 coverage tooling 정리

현재 `xlsx`의 active cell keyboard navigation과 multi-cell range selection은 hardening backlog가 아니라 Phase 3.6 delivered scope로 본다.

즉, 포맷 phase에서는 "출시 가능한 MVP + 실연동"을 우선 확보하고, hardening에서는 각 포맷에서 수집된 잔여 debt를 체계적으로 갚는 구조로 간다.

## Test corpus strategy
전략은 fixture 중심으로 간다.

### Selected approach
- hand-made fixture 중심
- 기능별 fixture
- 경계 케이스 fixture
- 암호화 문서 fixture
- 회귀용 고정 fixture 세트 유지

### Why
- 초기 프로젝트 단계에서 가장 관리가 쉽다.
- 기능 구현과 회귀 확인에 충분히 효과적이다.
- 이후 필요해지면 anonymized real-world corpus를 추가한다.

## MVP scope by format

### Phase 1: DOCX
- 단락, 런, 기본 스타일
- 제목/본문
- 리스트/번호 매기기
- 표 안 텍스트 렌더링
- 이미지
- 섹션 구분
- 헤더/푸터
- 페이지 단위 레이아웃
- 검색
- 순수 텍스트 선택

### Phase 1.5: DOCX demo real integration
- Rust FFI로 실제 DOCX page render model fetch
- Flutter bridge에서 실제 page fetch
- demo app fixture/file picker/desktop drop 경로에서 실제 DOCX 열기
- password-required demo flow 확인

### Phase 2: PPTX
- 슬라이드
- 슬라이드 크기와 배경
- 텍스트 박스
- 기본 도형
- 이미지
- 테마 색상/폰트의 기본 반영
- 슬라이드 내 검색
- 순수 텍스트 선택

제외 권장:
- 애니메이션
- 전환 효과
- 발표자 노트 고급 처리

### Phase 2.5: PPTX demo real integration
- Rust FFI로 실제 slide render model fetch
- Flutter bridge에서 실제 slide fetch
- demo app fixture/file picker/desktop drop 경로에서 실제 PPTX 열기

### Phase 3: XLSX
- 시트 렌더링
- 행/열 크기
- 셀 텍스트/숫자/기본 서식
- 병합 셀
- 고정 행/열
- 기본 테두리/배경색
- 검색
- 순수 텍스트 선택
- 수식 셀은 cached value 우선 표시
- 최종 UX는 page viewer가 아니라 `grid-first 2D spreadsheet viewer`로 간다

### Phase 3.5: XLSX demo real integration
- Rust FFI로 실제 sheet window render model fetch
- Flutter bridge에서 실제 sheet window fetch
- demo app fixture/file picker/desktop drop 경로에서 실제 XLSX 열기
- 단, 이 phase의 acceptance는 "단순 grid가 보인다"가 아니라 "sheet viewport로 탐색 가능하다"까지를 기준으로 삼는다

초기 제약 권장:
- 직접 수식 계산 미지원
- 캐시 없는 수식 셀은 후속 과제
- 차트, 피벗, 매크로, 외부 연결은 후순위

### XLSX viewer UX direction
- `XLSX`는 `DOCX/PPTX`와 달리 page-like preview를 최종 UX로 삼지 않는다.
- row/column header, 2D scroll, frozen pane, visible window virtualization을 가진 sheet viewport가 기본 모델이다.
- 초기 dense grid render model은 엔진 검증용 transitional step으로 보고, 실제 product UX는 별도 sheet viewport로 올린다.
- 세부 방향은 [docs/architecture/xlsx_viewer_ux.md](/Users/ultramarine/Documents/Workspace/Fapirus/ms_viewer/docs/architecture/xlsx_viewer_ux.md)를 따른다.

## FileHandle support
우선순위는 낮지만 구조는 초기에 반영하는 편이 좋다.
이유는 입력 추상화가 뒤늦게 바뀌면 파서 API 전체가 흔들리기 때문이다.

권장 입력 계층:
- `PathSource`
- `BytesSource`
- `FileHandleSource`

초기 MVP에서는 `Path`와 `Bytes`만 구현하고, 퍼블릭 API는 공통 추상화로 시작하는 편이 안전하다.

## Technical architecture

### Rust responsibilities
- OOXML package open/read
- XML parse
- format-specific model parse (`docx`, `pptx`, `xlsx`)
- style/theme resolve
- layout calculation
- search index generation
- text selection metadata generation
- encrypted package handling
- document/page/slide/sheet model serialization for FFI

### Flutter responsibilities
- viewport and scroll
- zoom
- gesture handling
- selection UI
- search UI
- password prompt UI
- page/slide/sheet navigation
- actual painting from engine output
- platform integration

## Engine output model
초기부터 아래 구조를 갖는 것이 좋다.
- document model
- render node tree or display list
- text spans with bounding boxes
- image/object references
- hit-test data
- search match ranges
- page or slide level layout boundaries

이 정보가 있어야 검색 하이라이트와 텍스트 선택을 Flutter에서 자연스럽게 구현할 수 있다.

## Remaining follow-up topics
이 항목들은 방향은 정했지만, 별도 세부 문서가 필요하다.

1. 플랫폼별 fallback font map 상세표
2. 암호화 문서 에러 코드와 UX 흐름
3. 캐시 eviction 파라미터
4. 수식 엔진 후속 과제 범위
5. Linux 지원성 검토 기준

## Recommended milestones

### Milestone 0: Foundations
- workspace 구성
- Rust core crate 생성
- Flutter plugin/package 생성
- FFI 브리지 생성
- 공통 document model 초안 작성
- archive-backed lazy loading 구조 초안 작성
- fixture/test corpus 구조 생성
- fallback font policy 초안 작성

### Milestone 1: DOCX viewer MVP
- OOXML unzip + XML parse
- paragraph/run/layout MVP
- 페이지 단위 레이아웃
- 이미지/표 기본 렌더링
- 검색
- 순수 텍스트 선택
- Flutter demo viewer

### Milestone 2: PPTX viewer MVP
- slide tree parse
- text box and basic shapes
- theme partial support
- 검색
- 순수 텍스트 선택

### Milestone 3: XLSX viewer MVP
- worksheet grid model
- cell style subset
- frozen panes
- cached value based formula display
- 검색
- 순수 텍스트 선택

### Milestone 3.5: Viewer Shell Refactor
- library screen / viewer screen 분리
- `DOCX/PPTX` 연속 스크롤 viewer
- `XLSX` full-screen viewport shell
- viewer chrome 정리
- format별 수동 검증 기준 재정의

### Milestone 4: Hardening
- encrypted document password flow
- FileHandle support
- performance optimization
- regression fixture and snapshot tests
- Linux feasibility review
- formula engine follow-up review

## Expected project structure
```text
ms_viewer/
  PROJECT_PLAN.md
  office-specs/
    README.md
    VIEWER_SPEC_RESEARCH.md
    standards/
      ecma-376/
        ...pdf
    microsoft/
      formats/
        MS-DOCX.pdf
        MS-PPTX.pdf
        MS-XLSX.pdf
      shared/
        MS-OI29500.pdf
        MS-ODRAWXML.pdf
        MS-OFFCRYPTO.pdf
        MS-OSHARED.pdf
  docs/
    architecture/
      ffi_contract.md
      render_model.md
      font_fallback_policy.md
      cache_policy.md
      crypto_flow.md
  packages/
    ms_viewer/
      lib/
        ms_viewer.dart
        src/
          controller/
          models/
          widgets/
          painting/
          selection/
          search/
          password/
      ios/
      android/
      macos/
      windows/
      linux/
      test/
    ms_viewer_platform_interface/
      lib/
      test/
  rust/
    Cargo.toml
    crates/
      viewer_core/
        src/
          archive/
          xml/
          model/
          layout/
          text/
          search/
          crypto/
          ffi/
          error/
          cache/
        tests/
      format_docx/
        src/
      format_pptx/
        src/
      format_xlsx/
        src/
      format_shared/
        src/
  ffi/
    bindings/
    generated/
  examples/
    flutter_demo/
      lib/
      test/
  fixtures/
    docx/
    pptx/
    xlsx/
    encrypted/
    regression/
  scripts/
    update_bindings.sh
    run_fixtures.sh
```

## Why this structure
- Flutter 패키지와 Rust 엔진의 경계를 명확히 유지할 수 있다.
- 포맷별 파서를 별도 crate로 두면 `docx -> pptx -> xlsx` 순서 개발이 자연스럽다.
- `format_shared`와 `viewer_core`를 분리하면 공통 기능 재사용이 쉬워진다.
- `crypto/`, `cache/`, `password/`를 분리하면 암호화 문서와 대용량 문서 흐름을 명확히 관리할 수 있다.
- docs 디렉터리를 분리하면 아직 구현 전인 정책 문서를 점진적으로 채워나가기 좋다.
- fixture와 example을 초기에 분리해두면 회귀 테스트와 데모 관리가 수월하다.

## Suggested next step
다음 단계는 `Milestone 0` 기준으로 API 초안을 먼저 잡는 것이다.
특히 아래 4개를 먼저 정의해야 한다.
- Rust document output schema
- Flutter rendering contract
- public open API (`path`, `bytes`, future `file handle`)
- font fallback and archive cache policy
