# OOXML Viewer Spec Research

## Conclusion
- 현재 컬렉션은 `docx`, `pptx`, `xlsx` 뷰어 조사와 설계에 필요한 필수 문서를 모두 포함한다.
- 추가 필수 문서는 없다.
- 유지보수 관점에서는 Microsoft 문서는 `Last-Modified`, ECMA 문서는 `Part + Edition` 기준으로 추적하는 것이 적절하다.

## Recommended structure
현재 구조를 아래처럼 정리했다.
- `office-specs/standards/ecma-376/`
- `office-specs/microsoft/formats/`
- `office-specs/microsoft/shared/`

이 구조를 권장하는 이유:
- ECMA 표준과 Microsoft 보완 스펙의 성격이 다르다.
- `docx/pptx/xlsx` 전용 문서와 공통 문서를 분리하면 탐색 속도가 빨라진다.
- 향후 갱신 시 어느 출처를 다시 확인해야 하는지 바로 보인다.

## Required documents

| Status | Document | Why needed | Local path |
|---|---|---|---|
| Required | ECMA-376 Part 1 | WordprocessingML, SpreadsheetML, PresentationML, DrawingML 본문 | `office-specs/standards/ecma-376/Ecma Office Open XML Part 1 - Fundamentals And Markup Language Reference.pdf` |
| Required | ECMA-376 Part 2 | OPC 패키징, 파트, relationship 구조 | `office-specs/standards/ecma-376/Ecma Office Open XML Part 2 - Open Packaging Conventions.pdf` |
| Required | ECMA-376 Part 3 | Markup Compatibility, `AlternateContent` 처리 | `office-specs/standards/ecma-376/Ecma Office Open XML Part 3 - Markup Compatibility and Extensibility.pdf` |
| Required | MS-DOCX | Word 문서의 Microsoft 확장 및 실제 제품 동작 | `office-specs/microsoft/formats/MS-DOCX.pdf` |
| Required | MS-PPTX | PowerPoint 문서의 Microsoft 확장 및 실제 제품 동작 | `office-specs/microsoft/formats/MS-PPTX.pdf` |
| Required | MS-XLSX | Excel 문서의 Microsoft 확장 및 실제 제품 동작 | `office-specs/microsoft/formats/MS-XLSX.pdf` |
| Required | MS-OI29500 | Office의 ISO/IEC 29500 지원 범위와 예외 | `office-specs/microsoft/shared/MS-OI29500.pdf` |
| Required | MS-ODRAWXML | DrawingML 확장 해석 보강 | `office-specs/microsoft/shared/MS-ODRAWXML.pdf` |
| Required for service-grade behavior | MS-OFFCRYPTO | 암호화/암호 보호 문서 감지 및 처리 | `office-specs/microsoft/shared/MS-OFFCRYPTO.pdf` |
| Recommended for service-grade behavior | MS-OSHARED | Office 공통 구조와 공유 동작 보강 | `office-specs/microsoft/shared/MS-OSHARED.pdf` |

## Optional document

| Status | Document | Why optional | Download link |
|---|---|---|---|
| Optional | ECMA-376 Part 4 | Transitional 문서 호환이 중요할 때만 우선순위 상승 | https://ecma-international.org/wp-content/uploads/ECMA-376-4_5th_edition_december_2016.zip |

## Version and maintenance catalog

### ECMA-376
| Document | Version basis | Download link | Official last modified | Local collected at |
|---|---|---|---|---|
| ECMA-376 Part 1 | 5th edition (December 2016) | https://ecma-international.org/wp-content/uploads/ECMA-376-1_5th_edition_december_2016.zip | 2022-11-02 16:19:11 GMT | 2026-03-07 01:32:14 +0900 |
| ECMA-376 Part 2 | 5th edition (December 2021) | https://ecma-international.org/wp-content/uploads/ECMA-376-2_5th_edition_december_2021.zip | 2022-11-02 16:21:19 GMT | 2026-03-07 01:32:30 +0900 |
| ECMA-376 Part 3 | 5th edition (December 2015) | https://ecma-international.org/wp-content/uploads/ECMA-376-3_5th_edition_december_2015.zip | 2022-11-02 16:23:10 GMT | 2026-03-07 01:32:51 +0900 |
| ECMA-376 Part 4 | 5th edition (December 2016) | https://ecma-international.org/wp-content/uploads/ECMA-376-4_5th_edition_december_2016.zip | 2022-11-02 16:26:17 GMT | 2026-03-07 01:32:36 +0900 |

### Microsoft Open Specifications
Microsoft 쪽은 문서 제목만으로 버전 추적이 어려우므로, 공식 배포 URL의 `Last-Modified`를 관리 기준으로 둔다.

| Document | Why tracked | Download link | Official last modified | Local collected at | Note |
|---|---|---|---|---|---|
| MS-DOCX | Word 확장 스펙 | https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-DOCX/%5BMS-DOCX%5D.pdf | 2025-11-14 08:54:27 GMT | 2026-03-07 01:30:51 +0900 | Current local size matches official content length |
| MS-PPTX | PowerPoint 확장 스펙 | https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-PPTX/%5BMS-PPTX%5D.pdf | 2025-06-03 20:27:07 GMT | 2026-03-07 01:31:08 +0900 | Local size 6978916 bytes, official Content-Length 5066218 bytes, refresh recommended |
| MS-XLSX | Excel 확장 스펙 | https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-XLSX/%5BMS-XLSX%5D.pdf | 2026-01-13 08:20:58 GMT | 2026-03-07 01:30:51 +0900 | Current local size matches official content length |
| MS-OI29500 | Office의 ISO/IEC 29500 지원 범위 | https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-OI29500/%5BMS-OI29500%5D.pdf | 2025-06-03 19:46:28 GMT | 2026-03-07 01:35:57 +0900 | Current local size matches official content length |
| MS-ODRAWXML | DrawingML 공통 확장 | https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-ODRAWXML/%5BMS-ODRAWXML%5D.pdf | 2026-02-16 06:59:33 GMT | 2026-03-07 01:38:15 +0900 | Current local size matches official content length |
| MS-OFFCRYPTO | 암호화 문서 처리 | https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-OFFCRYPTO/%5BMS-OFFCRYPTO%5D.pdf | 2026-02-16 07:07:32 GMT | 2026-03-07 01:48:44 +0900 | Current local size matches official content length |
| MS-OSHARED | Office 공통 동작 | https://officeprotocoldocs-f5hpbjgea6b8gneq.b02.azurefd.net/files/MS-OSHARED/%5BMS-OSHARED%5D.pdf | 2025-11-14 08:50:10 GMT | 2026-03-07 01:48:47 +0900 | Current local size matches official content length |

## Maintenance checklist
- Microsoft 문서는 `Last-Modified`가 바뀌면 새 PDF로 교체 검토
- ECMA 문서는 `Edition`이 바뀌면 우선 갱신 검토
- 새 파일을 받을 때는 기존 파일 크기와 공식 `Content-Length`를 함께 기록
- `MS-PPTX.pdf`는 현재 로컬 파일과 공식 배포본 크기가 달라 재다운로드 확인이 필요

## Link validation
2026-03-07 `Asia/Seoul` 기준으로 아래 링크의 유효성을 확인했다.
- `MS-DOCX`, `MS-PPTX`, `MS-XLSX`, `MS-OI29500`, `MS-ODRAWXML`, `MS-OFFCRYPTO`, `MS-OSHARED` 공식 PDF 링크: `200 OK`
- ECMA-376 Part 1, 2, 3, 4 ZIP 링크: `200 OK`
- Microsoft Learn 랜딩 페이지와 ECMA overview 페이지: 접근 가능 확인

## Implementation notes
- `docx`: 본문, 스타일, 번호 매기기, 표, 이미지, 섹션, 헤더/푸터 순서로 분석하는 것이 효율적이다.
- `pptx`: 슬라이드, 레이아웃, 마스터, 테마까지를 MVP에 포함할지 먼저 정해야 한다.
- `xlsx`: 수식 계산 엔진을 구현할지, 캐시된 셀 결과만 렌더링할지 범위를 먼저 고정해야 한다.
- 암호화 문서를 지원하지 않더라도 `MS-OFFCRYPTO` 기준으로 감지 후 명확한 오류 메시지를 제공하는 편이 좋다.
