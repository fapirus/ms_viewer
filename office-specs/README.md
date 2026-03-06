# Office Specs

## Current status
- `docx`, `pptx`, `xlsx` 뷰어 스펙 조사에 필요한 필수 문서는 수집 완료 상태다.
- 유지보수 기준 문서는 `VIEWER_SPEC_RESEARCH.md` 하나로 관리한다.

## Folder structure
- `standards/ecma-376/`: ECMA-376 표준 본문 PDF
- `microsoft/formats/`: 형식별 Microsoft Open Specifications (`MS-DOCX`, `MS-PPTX`, `MS-XLSX`)
- `microsoft/shared/`: Office 공통 Microsoft Open Specifications (`MS-OI29500`, `MS-ODRAWXML`, `MS-OFFCRYPTO`, `MS-OSHARED`)

## Why this structure
- 표준 원문과 Microsoft 구현 보완 문서를 분리해서 찾기 쉽다.
- 형식별 문서와 공통 문서를 나눠서 참조 우선순위가 명확하다.
- 유지보수 시 "표준 문서 갱신"과 "Microsoft 배포본 갱신"을 별도로 추적할 수 있다.

## Maintenance rule
- 새 문서를 추가할 때는 먼저 `VIEWER_SPEC_RESEARCH.md`에 목적, 링크, 버전 또는 업데이트 기준을 기록한다.
- Microsoft Open Specifications는 문서 번호보다 `Last-Modified`가 실질적인 추적 기준이다.
- ECMA-376은 `Part`와 `Edition`을 우선 버전 기준으로 본다.
