# Fixtures

이 디렉터리는 문서 뷰어 검증용 fixture를 관리한다.

## Structure
- `docx/`: DOCX 기능별 fixture
- `pptx/`: PPTX 기능별 fixture
- `xlsx/`: XLSX 기능별 fixture
- `encrypted/`: 암호화 문서 fixture
- `regression/`: 회귀 테스트 고정 세트

## Naming
파일명은 기능이 드러나게 짓는다.
예:
- `docx_basic_paragraphs.docx`
- `docx_nested_lists.docx`
- `pptx_text_and_shapes.pptx`
- `xlsx_frozen_panes.xlsx`

## Rule
- 새 기능을 구현하면 최소 1개의 fixture를 추가한다.
- 버그를 고치면 해당 버그를 재현하는 regression fixture를 추가한다.
- fixture는 가능한 작고 단순하게 유지한다.

## Current review sets
- DOCX Phase 1 review set:
  - `docx/docx_plain_text.docx`
  - `docx/docx_styles_lists.docx`
  - `docx/docx_tables_images.docx`
  - `docx/docx_multi_section.docx`
  - `encrypted/docx_password_stub.docx`
  - `regression/docx_acceptance_plain_text.docx`
- PPTX Phase 2 review set:
  - `pptx/pptx_text_shapes.pptx`
  - `pptx/pptx_theme_layout_images.pptx`
- XLSX Phase 3 review set:
  - `xlsx/xlsx_basic_grid.xlsx`
  - `xlsx/xlsx_merges_frozen_formulas.xlsx`
