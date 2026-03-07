# DOCX Visual Triage

이 문서는 `issue/word/*`에 저장한 실제 문서와 Word/Viewer 비교 스크린샷을 기준으로
현재 DOCX 시각 회귀를 분류한 기록이다.

## 분석 규칙
- `issue/`는 로컬 분석용 원본과 스크린샷을 보관한다.
- 원인이 고정되면 최소 재현 문서를 `fixtures/regression/`로 옮겨 자동 테스트에 편입한다.
- 한 이슈는 `증상 -> 원인 가설 -> 최소 재현 fixture 후보 -> 체크리스트 항목` 순서로 관리한다.

## 우선순위 표

| Case | 기준 자료 | 현재 증상 | 원인 가설 | 우선순위 | 난이도 | 체크리스트 항목 | 최소 재현 fixture 후보 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `hello` | `issue/word/hello/1.png`, `2.png`, viewer 스크린샷 2장 | 페이지 1/2 경계가 Word와 다르고, 페이지 2 텍스트가 과소 렌더된다 | `w:lastRenderedPageBreak` 힌트를 무시하고 자체 줄배치만으로 페이지를 나눔. carry-over height와 빈 문단 spacing도 함께 영향 | P0 | 높음 | `DOCX 페이지 단위 계산 보정` | `docx_rendered_page_break.docx` |
| `infinity` page 1 | `issue/word/infinity/1.png`, viewer page 1 스크린샷 | 표 시작 위치, 표 크기, 표 하단 이후 문단 흐름이 Word와 다르다 | `w:tblpPr`가 있는 floating table을 inline table처럼 처리하고 있음. preferred width, table position, cell padding 반영 부족 | P0 | 높음 | `DOCX 표 크기와 셀 내부 줄바꿈 보정` | `docx_floating_table_intro.docx` |
| `infinity` page 4-5 | `issue/word/infinity/4.png`, `5.png`, viewer page 4-5 스크린샷 | 표 셀 내부 이미지가 빠지고 텍스트만 좁게 배치된다 | 표 셀 내부 `w:drawing`을 cell text flatten 과정에서 버리고 있음. row height도 이미지 크기를 반영하지 않음 | P0 | 높음 | `DOCX 표 내부 이미지 및 inline image 렌더 보정` | `docx_table_with_inline_images.docx` |
| `infinity` 전체 | viewer page 2-8 스크린샷 | 문단 간격과 표/문단 사이 여백이 Word보다 빽빽하다 | `before/after`, line spacing, style paragraph metrics를 대부분 기본값으로 처리 | P1 | 중상 | `DOCX 문단 간격과 기본 스타일 메트릭 보정` | `docx_spacing_variants.docx` |
| `hello`, `infinity` 공통 | viewer 스크린샷 전반 | 글자폭과 줄바꿈이 Word와 완전히 일치하지 않는다 | 추정 문자폭 기반 line breaking 한계. 실제 폰트 메트릭과 fallback 정밀도가 부족 | P2 | 매우 높음 | `DOCX 폰트 메트릭과 fallback 정밀도 보정` | `docx_cjk_width_mix.docx` |

## 현재 결론
- `Phase 1.6`의 첫 번째 실제 수정 우선순위는 `hello` 케이스의 페이지 경계 보정이다.
- 그 다음은 `infinity`의 floating table과 table-embedded image 처리다.
- 간격과 폰트 정밀도는 앞의 구조적 문제를 닫은 뒤 다루는 편이 맞다.

## 추가 제안
- `issue/word/JINWOOK/` 케이스를 같은 형식으로 추가하는 편이 좋다.
- 케이스별로 `notes.md`를 두고 아래 항목만 기록하면 충분하다.
  - Word 기준 기대 결과
  - 현재 viewer 결과
  - 재현 절차
  - 의심 원인
  - 우선순위
