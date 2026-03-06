# ms_viewer

Rust 엔진과 Flutter UI를 결합한 Microsoft Office 문서 뷰어 프로젝트다.
현재는 프로젝트 골격과 기획 문서가 준비된 상태이며, 구현은 `docx -> pptx -> xlsx` 순서로 진행한다.

## Workspace layout
- `PROJECT_PLAN.md`: 제품 목표와 단계별 계획
- `office-specs/`: 수집한 OOXML 및 Microsoft Open Specifications 문서
- `docs/architecture/`: 구현 전에 고정해야 할 세부 정책 문서
- `packages/ms_viewer/`: Flutter 패키지
- `packages/ms_viewer_platform_interface/`: Flutter 플랫폼 인터페이스
- `rust/`: Rust 워크스페이스와 엔진 crate
- `examples/flutter_demo/`: 샘플 앱
- `fixtures/`: 테스트용 문서 fixture

## Flutter SDK
이 저장소는 `fvm` 기준 Flutter `3.41.4`를 사용한다.

## Bootstrap
1. `fvm use 3.41.4`
2. `fvm flutter pub get` in `packages/ms_viewer`
3. `fvm flutter pub get` in `packages/ms_viewer_platform_interface`
4. `fvm flutter pub get` in `examples/flutter_demo`
5. `cargo check --manifest-path rust/Cargo.toml`

## Development tracking
- `DEVELOPMENT_CHECKLIST.md`: 단계별 체크리스트와 테스트 절차

## First implementation target
- `docx` parsing MVP
- 페이지 단위 레이아웃 모델
- 검색 메타데이터
- 순수 텍스트 선택 메타데이터
